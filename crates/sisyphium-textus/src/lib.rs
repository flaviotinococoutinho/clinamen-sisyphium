//! Native terminal presentation and independent monotonic cadences (REQ-H-02, REQ-C-05).
pub mod signa;
pub mod terminalis;

use sisyphium::{
    hospes::{Cellae, Facultates, Nuntius, Regimen, Tabula},
    mundus::Mundus,
};
use sisyphium_hospes::{
    Error, MAX_CELLS, MAX_PIXELS,
    archivum::Archivum,
    argumenta::{Argumenta, Communia},
    electio::Electio,
    tempus::{elapsed_ticks, epoch_ms},
    validate_area,
};
use std::{
    ffi::OsString,
    io::{self, IsTerminal, Write},
    str::FromStr,
    thread,
    time::{Duration, Instant, SystemTime},
};

/// Documented native options; common options are recognized by Argumenta (REQ-H-01).
const OPTIONS: &[(&str, bool)] = &[
    ("--cols", true),
    ("--rows", true),
    ("--cells", true),
    ("--frames", true),
    ("--plain", false),
    ("--max-hz", true),
    ("--help", false),
    ("--version", false),
];
/// Public usage includes every supported common and terminal option (REQ-H-01).
const HELP: &str = "sisyphium-textus [options]\n\
--cols <1..65535> --rows <1..65535>  usable grid dimensions\n\
--cells <true|false> (true) --plain --frames <1..u32::MAX>\n\
--max-hz <0..24> (24); zero presents at most once per minute\n\
--epoch <i64-ms> requires --ticks and --frames 1\n\
--ticks <0..65536> --seed <u64> --state <path>\n\
--energy <u8> (255) --seism <u8> (0) --lingua <en|pt> (en)\n\
--compute <u8> (255) --color-bits <1|4|8|24> (8)\n\
--tz-offset <-1439..1439> (-180) --australis <true|false> (true)\n\
--help --version\n\
Pipes and unsupported terminals default to one plain frame.\n\
No input, cursor visibility, alternate screen or console mode is changed.\n";

/// Parsed terminal choices, with physical dimensions resolved later (REQ-H-01).
struct Optiones {
    communia: Communia,
    columnae: Option<u16>,
    lineae: Option<u16>,
    cellae: bool,
    plana: bool,
    finita: Option<u32>,
    frequentia: u8,
}
impl Optiones {
    /// Consumes options and rejects synthetic animation before effects (REQ-H-06).
    fn parse(mut args: Argumenta) -> io::Result<Self> {
        let common = Communia::parse(&mut args)?;
        let cols = number(&mut args, "--cols")?;
        let rows = number(&mut args, "--rows")?;
        let cells = number(&mut args, "--cells")?.unwrap_or(true);
        let frames = number(&mut args, "--frames")?;
        let max_hz = number(&mut args, "--max-hz")?.unwrap_or(24);
        let plain = args.take("--plain").is_some() || !ansi_capable();
        if cols == Some(0) || rows == Some(0) || frames == Some(0) || max_hz > 24 {
            return Err(invalid(
                "invalid terminal dimensions, frame count or maximum frequency",
            ));
        }
        if common.epocha.is_some() && frames != Some(1) {
            return Err(invalid("--epoch requires --frames 1"));
        }
        args.finish()?;
        Ok(Self {
            communia: common,
            columnae: cols,
            lineae: rows,
            cellae: cells,
            plana: plain,
            finita: frames.or(if plain { Some(1) } else { None }),
            frequentia: max_hz,
        })
    }

    /// Resolves native geometry once initially and at cached presentations (REQ-H-02).
    fn size(&self, probe: Option<(u16, u16)>) -> io::Result<(u16, u16)> {
        terminalis::resolve_size(
            self.columnae,
            self.lineae,
            probe,
            environment_number("COLUMNS"),
            environment_number("LINES"),
        )
    }

    /// Explicit complete dimensions avoid all native subprocess probes (REQ-H-02).
    fn needs_probe(&self) -> bool {
        self.columnae.is_none() || self.lineae.is_none()
    }
}

/// Allocated surfaces and reusable encoded frame, all bounded before state access (REQ-H-02).
struct Superficies {
    columnae: u16,
    lineae: u16,
    altitudo: u16,
    pictura: Vec<u8>,
    cellae: Vec<u16>,
    textus: Vec<u8>,
}
impl Superficies {
    /// Checks geometry and reserves every output buffer before evolution (REQ-H-02).
    fn new(cols: u16, rows: u16, cells: bool, plain: bool) -> io::Result<Self> {
        let height = raster_height(rows, cells)?;
        let area = validate_area(cols, height, MAX_PIXELS)?;
        let cell_count = validate_area(cols, rows, MAX_CELLS)?;
        Ok(Self {
            columnae: cols,
            lineae: rows,
            altitudo: height,
            textus: terminalis::reserve_frame(cols, rows, plain)?,
            pictura: zeroed(area)?,
            cellae: zeroed(if cells { cell_count } else { 0 })?,
        })
    }

    /// Renders native cells with a same-size scratch, or twice-height raster (REQ-H-02).
    fn render(
        &mut self,
        world: &Mundus,
        host: &Facultates,
        indications: &sisyphium::hospes::Indicia,
        regimen: &Regimen,
        plain: bool,
        subsequent: bool,
    ) -> io::Result<()> {
        self.textus.clear();
        if subsequent {
            self.textus
                .extend_from_slice(if plain { b"\n" } else { b"\x1b[H" });
        }
        let palette = world.palette(host, indications, regimen);
        let mut raster = Tabula::new(self.columnae, self.altitudo, &mut self.pictura)
            .ok_or_else(|| invalid("invalid scratch surface"))?;
        if host.cells {
            let mut cells = Cellae::new(self.columnae, self.lineae, &mut self.cellae)
                .ok_or_else(|| invalid("invalid cell surface"))?;
            if !world.render_cellae(host, indications, regimen, &mut raster, &mut cells) {
                return Err(invalid("native cell renderer rejected scratch geometry"));
            }
            terminalis::encode_cells(
                &mut self.textus,
                &self.cellae,
                self.columnae,
                self.lineae,
                &palette,
                plain,
            )
        } else {
            world.render(host, indications, regimen, &mut raster);
            terminalis::encode_blocks(
                &mut self.textus,
                &self.pictura,
                self.columnae,
                self.altitudo,
                &palette,
                plain,
            )
        }
    }
}

/// Monotonic deadlines independent of civil epoch values (REQ-H-06).
#[derive(Debug)]
struct Cadentia {
    ictus: u64,
    imago: u64,
}
impl Cadentia {
    /// Presents immediately and starts evolution one full tick later (REQ-H-06).
    fn new(tick_ms: u32) -> Self {
        Self {
            ictus: u64::from(tick_ms.max(1)),
            imago: 0,
        }
    }
    /// Advances by the newly selected period; the caller bounds each catch-up batch (REQ-C-05).
    fn tick(&mut self, tick_ms: u32) {
        self.ictus = self.ictus.saturating_add(u64::from(tick_ms.max(1)));
    }
    /// Drops excess backlog at the finite catch-up limit, avoiding an immediate busy retry (REQ-H-06).
    fn cap(&mut self, now: u64, tick_ms: u32) {
        if self.ictus <= now {
            self.ictus = now.saturating_add(u64::from(tick_ms.max(1)));
        }
    }
    /// Schedules the next presentation from completion, without replaying missed frames (REQ-H-02).
    fn presented(&mut self, now: u64, interval_ms: u32) {
        self.imago = now.saturating_add(u64::from(interval_ms.max(1)));
    }
    /// Returns a nonnegative sleep capped to 25 ms for cooperative signal checks (REQ-D-02).
    fn wait(&self, now: u64) -> Duration {
        Duration::from_millis(self.ictus.min(self.imago).saturating_sub(now).min(25))
    }
}

/// Runs the terminal host with arguments excluding the executable name (REQ-H-01).
///
/// # Errors
/// Classifies malformed options as usage errors and output, allocation, signal or state
/// failures as operational errors. Output failure attempts normal-context persistence.
pub fn run(args: &[OsString], output: &mut dyn Write) -> Result<(), Error> {
    let mut parsed = Argumenta::parse(args, OPTIONS).map_err(Error::Usus)?;
    if parsed.take("--help").is_some() {
        return output
            .write_all(HELP.as_bytes())
            .and_then(|()| output.flush())
            .map_err(Error::Operatio);
    }
    if parsed.take("--version").is_some() {
        return writeln!(output, "sisyphium-textus {}", env!("CARGO_PKG_VERSION"))
            .and_then(|()| output.flush())
            .map_err(Error::Operatio);
    }
    let options = Optiones::parse(parsed).map_err(Error::Usus)?;
    let measured = if options.needs_probe() {
        terminalis::measure()
    } else {
        None
    };
    let (cols, rows) = options.size(measured).map_err(Error::Usus)?;
    let height = raster_height(rows, options.cellae).map_err(Error::Usus)?;
    validate_area(cols, height, MAX_PIXELS).map_err(Error::Usus)?;
    terminalis::frame_bound(cols, rows, options.plana).map_err(Error::Usus)?;
    let mut surface =
        Superficies::new(cols, rows, options.cellae, options.plana).map_err(Error::Operatio)?;
    let guard = if options.finita == Some(1) {
        None
    } else {
        Some(signa::Custodia::install().map_err(Error::Operatio)?)
    };
    let result = execute(&options, &mut surface, measured, output);
    let restored = guard.map_or(Ok(()), signa::Custodia::restore);
    combine(result, restored, "signal restoration").map_err(Error::Operatio)
}

/// One state handle whose failed transaction must never be retried (REQ-D-02).
struct Persistentia {
    archivum: Option<Archivum>,
    fracta: bool,
    #[cfg(test)]
    scriptiones: usize,
}

/// Owns the state session and guarantees a single exit-save attempt on loop return (REQ-D-02).
fn execute(
    options: &Optiones,
    surface: &mut Superficies,
    measured: Option<(u16, u16)>,
    output: &mut dyn Write,
) -> io::Result<()> {
    let now = civil_time(options)?;
    let seed = options
        .communia
        .semen
        .unwrap_or(u64::from_ne_bytes(now.to_ne_bytes()));
    let (archive, mut world) = match &options.communia.archivum {
        Some(path) => {
            let (archive, world) = Archivum::open(path, options.communia.semen, seed)?;
            (Some(archive), world)
        }
        None => (None, Mundus::genesis(seed)),
    };
    let mut archive = Persistentia {
        archivum: archive,
        fracta: false,
        #[cfg(test)]
        scriptiones: 0,
    };
    let mut host = options.communia.facultates(
        surface.columnae,
        surface.altitudo,
        options.cellae,
        options.frequentia,
    );
    let mut choice = Electio::new();
    let regimen = choice.select(&host, &world, options.communia.energia);
    let ticks = options.communia.ictus.unwrap_or_else(|| {
        elapsed_ticks(
            now,
            archive
                .archivum
                .as_ref()
                .and_then(Archivum::mtime)
                .and_then(epoch_ms),
            regimen.tick_ms,
        )
    });
    let result = (|| {
        let notices = world.advance(
            &host,
            &options.communia.indicia(now),
            ticks,
            regimen.tick_ms,
        );
        let regimen = choice.select(&host, &world, options.communia.energia);
        save_milestone(&mut archive, &world, notices)?;
        animate(
            options,
            surface,
            measured,
            output,
            &mut world,
            &mut archive,
            &mut host,
            &mut choice,
            regimen,
        )
    })();
    combine(result, save(&mut archive, &world), "exit persistence")
}

/// Interleaves independently scheduled ticks and frames, caching native dimensions (REQ-C-05).
#[allow(clippy::too_many_arguments)]
fn animate(
    options: &Optiones,
    surface: &mut Superficies,
    mut measured: Option<(u16, u16)>,
    output: &mut dyn Write,
    world: &mut Mundus,
    archive: &mut Persistentia,
    host: &mut Facultates,
    choice: &mut Electio,
    mut regimen: Regimen,
) -> io::Result<()> {
    let origin = Instant::now();
    let mut cadence = Cadentia::new(regimen.tick_ms);
    let mut last_probe = 0;
    let mut frames = 0_u32;
    let mut last_frame = 0;
    loop {
        if signa::requested() {
            break;
        }
        let now = milliseconds(origin.elapsed());
        let mut catchup = 0;
        while cadence.ictus <= now && catchup < 65_536 && !signa::requested() {
            let _ = choice.select(host, world, options.communia.energia);
            let notices = world.step(host, &options.communia.indicia(civil_time(options)?));
            regimen = choice.select(host, world, options.communia.energia);
            save_milestone(archive, world, notices)?;
            if frames > 0 {
                cadence.presented(last_frame, regimen.interval_ms);
            }
            cadence.tick(regimen.tick_ms);
            catchup += 1;
        }
        if catchup == 65_536 {
            cadence.cap(milliseconds(origin.elapsed()), regimen.tick_ms);
        }
        if signa::requested() {
            break;
        }
        if now >= cadence.imago {
            if options.needs_probe() && now.saturating_sub(last_probe) >= 1_000 {
                measured = terminalis::measure().or(measured);
                last_probe = now;
                let (cols, rows) = options.size(measured)?;
                if (cols, rows) != (surface.columnae, surface.lineae) {
                    *surface = Superficies::new(cols, rows, options.cellae, options.plana)?;
                    *host = options.communia.facultates(
                        surface.columnae,
                        surface.altitudo,
                        options.cellae,
                        options.frequentia,
                    );
                    regimen = choice.select(host, world, options.communia.energia);
                }
            }
            surface.render(
                world,
                host,
                &options.communia.indicia(civil_time(options)?),
                &regimen,
                options.plana,
                frames > 0,
            )?;
            output.write_all(&surface.textus)?;
            output.flush()?;
            frames = frames.saturating_add(1);
            if options.finita.is_some_and(|limit| frames >= limit) {
                break;
            }
            last_frame = milliseconds(origin.elapsed());
            cadence.presented(last_frame, regimen.interval_ms);
        }
        let wait = cadence.wait(milliseconds(origin.elapsed()));
        if !wait.is_zero() {
            thread::sleep(wait);
        }
    }
    Ok(())
}

/// Persists only named milestones; a mere presentation never calls this law (REQ-D-02).
fn save_milestone(archive: &mut Persistentia, world: &Mundus, notices: Nuntius) -> io::Result<()> {
    if milestone(notices) {
        save(archive, world)
    } else {
        Ok(())
    }
}

/// Identifies the three persistence announcements independently of frame count (REQ-D-02).
fn milestone(notices: Nuntius) -> bool {
    [Nuntius::CLAUSURA, Nuntius::CLINAMEN, Nuntius::AGNITIO]
        .into_iter()
        .any(|notice| notices.contains(notice))
}

/// Saves only an explicitly opened state destination (REQ-D-02).
fn save(archive: &mut Persistentia, world: &Mundus) -> io::Result<()> {
    if archive.fracta {
        return Ok(());
    }
    #[cfg(test)]
    if archive.archivum.is_some() {
        archive.scriptiones += 1;
    }
    let result = archive
        .archivum
        .as_mut()
        .map_or(Ok(()), |archive| archive.save(world));
    archive.fracta = result.is_err();
    result
}

/// Retains the primary operational failure and any cleanup diagnostic (REQ-D-02).
fn combine(result: io::Result<()>, cleanup: io::Result<()>, context: &str) -> io::Result<()> {
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; {context}: {cleanup}"),
        )),
        (Err(error), _) | (_, Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

/// Keeps civil measurements separate from monotonic pacing (REQ-H-06).
fn civil_time(options: &Optiones) -> io::Result<i64> {
    options.communia.epocha.map_or_else(
        || {
            epoch_ms(SystemTime::now())
                .ok_or_else(|| io::Error::other("civil epoch is outside i64 milliseconds"))
        },
        Ok,
    )
}

/// Saturates elapsed milliseconds only beyond the u64 monotonic domain (REQ-H-06).
fn milliseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Checks the raster's doubled height without truncating a terminal row (REQ-H-02).
fn raster_height(rows: u16, cells: bool) -> io::Result<u16> {
    if cells {
        Ok(rows)
    } else {
        rows.checked_mul(2)
            .ok_or_else(|| invalid("raster height overflow"))
    }
}

/// Reserves bounded surface storage fallibly before zero initialization (REQ-H-02).
fn zeroed<T: Default + Clone>(count: usize) -> io::Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(io::Error::other)?;
    values.resize(count, T::default());
    Ok(values)
}

/// Parses one optional scalar without interpreting non-UTF8 bytes (REQ-H-01).
fn number<T: FromStr>(args: &mut Argumenta, name: &str) -> io::Result<Option<T>> {
    args.take(name)
        .map(|value| {
            value
                .to_str()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| invalid("invalid terminal option value"))
        })
        .transpose()
}

/// Reads a positive optional physical terminal dimension (REQ-H-02).
fn environment_number(name: &str) -> Option<u16> {
    std::env::var(name)
        .ok()?
        .parse()
        .ok()
        .filter(|&value| value > 0)
}

/// Requires a real TTY and declared ANSI support, without changing console mode (REQ-H-02).
fn ansi_capable() -> bool {
    if !io::stdout().is_terminal() {
        return false;
    }
    let term = std::env::var("TERM").is_ok_and(|term| !term.is_empty() && term != "dumb");
    #[cfg(all(windows, target_has_atomic = "8"))]
    {
        (term || std::env::var_os("WT_SESSION").is_some()) && signa::ansi_enabled()
    }
    #[cfg(not(windows))]
    {
        term
    }
    #[cfg(all(windows, not(target_has_atomic = "8")))]
    {
        false
    }
}

/// Constructs a usage-boundary diagnostic (REQ-H-01).
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// A long presentation interval does not suppress the world's tick clock (REQ-H-06).
    #[test]
    fn monotonic_deadlines() {
        let mut cadence = Cadentia::new(42);
        assert_eq!(cadence.imago, 0);
        cadence.presented(0, 60_000);
        assert_eq!(cadence.wait(0), Duration::from_millis(25));
        let mut ticks = 0;
        while cadence.ictus <= 1_000 {
            ticks += 1;
            cadence.tick(42);
        }
        assert_eq!(ticks, 23);
        assert_eq!(cadence.imago, 60_000);
        let mut caught_up = Cadentia::new(42);
        let elapsed = 42 * 100_000;
        let mut processed = 0;
        while caught_up.ictus <= elapsed && processed < 65_536 {
            caught_up.tick(42);
            processed += 1;
        }
        assert_eq!(processed, 65_536);
        caught_up.cap(elapsed, 42);
        assert_eq!(caught_up.ictus, elapsed + 42);
        caught_up.tick(60_000);
        assert_eq!(caught_up.ictus, elapsed + 42 + 60_000);
        cadence.cap(u64::MAX - 100, 42);
        assert!(cadence.ictus > u64::MAX - 100);
        assert_eq!(cadence.wait(u64::MAX), Duration::ZERO);
    }
    /// Rendering does not save; each named milestone and controlled exit does (REQ-D-02).
    #[test]
    fn milestones_and_exit_persist() -> io::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "sisyphium-textus-milestones-{}",
            std::process::id()
        ));
        std::fs::create_dir(&path)?;
        let state_path = path.join("world");
        let (archive, mut world) = Archivum::open(&state_path, Some(17), 17)?;
        let mut persistence = Persistentia {
            archivum: Some(archive),
            fracta: false,
            scriptiones: 0,
        };
        let host = Facultates {
            width: 4,
            height: 2,
            cells: true,
            ..Facultates::default()
        };
        let indications = sisyphium::hospes::Indicia::default();
        let regimen = Electio::new().select(&host, &world, 255);
        let mut surface = Superficies::new(4, 2, true, true)?;
        for subsequent in [false, true] {
            surface.render(&world, &host, &indications, &regimen, true, subsequent)?;
        }
        save_milestone(&mut persistence, &world, Nuntius::NIHIL)?;
        assert_eq!(persistence.scriptiones, 0);
        assert!(!state_path.exists());
        for (index, notice) in [Nuntius::CLAUSURA, Nuntius::CLINAMEN, Nuntius::AGNITIO]
            .into_iter()
            .enumerate()
        {
            world.verbum = world.verbum.with_semen(index as u16 + 20);
            save_milestone(&mut persistence, &world, notice)?;
            assert_eq!(std::fs::read(&state_path)?, world.format());
            assert_eq!(persistence.scriptiones, index + 1);
        }
        save(&mut persistence, &world)?;
        assert_eq!(persistence.scriptiones, 4);
        std::fs::remove_file(&state_path)?;
        std::fs::remove_dir(&path)?;
        Ok(())
    }

    /// Only the three designated announcements authorize milestone persistence (REQ-D-02).
    #[test]
    fn milestones_are_not_frames() {
        for notice in [Nuntius::CLAUSURA, Nuntius::CLINAMEN, Nuntius::AGNITIO] {
            assert!(milestone(notice));
        }
        for notice in [
            Nuntius::NIHIL,
            Nuntius::PERSONA,
            Nuntius::RARUM,
            Nuntius::LAPSUS,
            Nuntius::METALEPSIS,
        ] {
            assert!(!milestone(notice));
        }
    }
}
