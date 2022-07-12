use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use histograms::Histograms;
use jemalloc_context::JemallocCtlContext;
use jemallocator::Jemalloc;
use mibs::MIBs;
use ringbuffer::RingBuffer;
use snapshots::Snapshots;
use tui::backend::CrosstermBackend;
use tui::style::{Color, Style};
use tui::symbols::Marker;
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use tui::{symbols, Terminal};

mod histograms;
mod jemalloc_context;
mod mibs;
mod snapshots;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    App::new()?.run();

    Ok(())
}

struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    mibs: MIBs,
    histograms: Histograms,
    snapshots: Snapshots,
    allocs: Vec<Vec<u8>>,
}

impl App {
    fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        crossterm::terminal::enable_raw_mode()?;

        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("failed to create terminal")?;

        let histogram_capacity = 512;

        Ok(Self {
            terminal,
            mibs: MIBs::new().context("failed to create MIBs")?,
            histograms: Histograms::with_capacity(histogram_capacity).context("failed to create histogram")?,
            snapshots: Snapshots::new(),
            allocs: Vec::new(),
        })
    }

    fn update(&mut self) -> Result<()> {
        self.mibs
            .epoch
            .advance()
            .context("failed to advance epoch")?;

        self.histograms
            .sample(&self.mibs)
            .context("failed to sample MIBs")?;

        self.snapshots.snapshot_histograms(&self.histograms);

        Ok(())
    }

    fn create_datasets(snapshots: &Snapshots) -> Vec<Dataset> {
        let dataset = |name: &'static str, data| {
            Dataset::default()
                .name(name)
                .data(data)
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
        };

        vec![
            dataset("active", &snapshots.active).style(Style::default().fg(Color::Red)),
            dataset("allocated", &snapshots.allocated).style(Style::default().fg(Color::Blue)),
            dataset("mapped", &snapshots.mapped).style(Style::default().fg(Color::Green)),
            dataset("metadata", &snapshots.metadata).style(Style::default().fg(Color::Magenta)),
            dataset("retained", &snapshots.retained).style(Style::default().fg(Color::Yellow)),
        ]
    }

    fn render(&mut self) -> Result<()> {
        let datasets = Self::create_datasets(&self.snapshots);

        let max_mib = (self.snapshots.max() / 1024 / 1024).max(10);
        let max_y = max_mib as f64 * 1024. * 1024. + 1024.;

        let block = Block::default()
            .title(format!(
                "num samples: {}, max value: {:?}",
                self.snapshots.active.len(),
                self.snapshots.active.iter().map(|(_, v)| *v as u64).max()
            ))
            .borders(Borders::ALL);

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(
                Axis::default().bounds([0., self.histograms.capacity() as f64]), // .style(Style::default().fg(Color::Gray)),
            )
            .y_axis(
                Axis::default()
                .bounds([0., max_y])
                .labels((0..=max_mib).step_by(2).map(|size| Span::from(format!("{size} MiB"))).collect())
            );

        self.terminal
            .draw(|f| {
                let size = f.size();
                f.render_widget(chart, size);
            })
            .context("drawing to terminal failed")?;

        Ok(())
    }

    fn run(mut self) -> Result<()> {
        loop {
            self.render()?;

            let event_available =
                || event::poll(Duration::from_millis(1000 / 60)).context("failed to poll events");

            while event_available()? {
                let event = event::read()?;

                match event {
                    Event::Key(KeyEvent { code, modifiers }) => match code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char(char @ '0'..='9') => {
                            let mut size_mib = match char {
                                '1' => 10,
                                '2' => 20,
                                '3' => 30,
                                '4' => 40,
                                '5' => 50,
                                '6' => 60,
                                '7' => 70,
                                '8' => 80,
                                '9' => 90,
                                '0' => 100,
                                _ => unreachable!(),
                            };

                            if modifiers.contains(KeyModifiers::SHIFT) {
                                size_mib *= 10;
                            }

                            let size_bytes = size_mib * 1024 * 1024;
                            let alloc = match modifiers.contains(KeyModifiers::CONTROL) {
                                false => Vec::with_capacity(size_bytes),
                                true => vec![42; size_bytes],
                            };

                            self.allocs.push(alloc);
                        },
                        KeyCode::Char('p') => {
                            self.allocs.pop();
                        }
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    },
                    Event::Mouse(_) => {}
                    Event::Resize(_, _) => {}
                }
            }

            self.update()?;
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().ok();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).ok();
    }
}
