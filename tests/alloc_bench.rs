//! Allocation-counting benchmark for the per-frame rendering pipeline.
//!
//! Simulates the hot path of the main loop (dimming update, visual lines,
//! WritingSurface build + render) and counts heap allocations per frame.
//!
//! Run with: cargo test --test alloc_bench -- --nocapture

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Counting allocator wrapper around System.
struct CountingAlloc;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static COUNTING_ACTIVE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING_ACTIVE.load(Ordering::Relaxed) > 0 {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn start_counting() {
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    COUNTING_ACTIVE.store(1, Ordering::SeqCst);
}

fn stop_counting() -> usize {
    COUNTING_ACTIVE.store(0, Ordering::SeqCst);
    ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Generate a realistic ~200-line markdown document for benchmarking.
fn sample_document() -> String {
    let mut doc = String::new();
    doc.push_str("# Chapter One\n\n");
    for i in 0..30 {
        doc.push_str(&format!(
            "This is paragraph {}. It contains several sentences of prose. \
             The quick brown fox jumps over the lazy dog. Writing is a craft \
             that requires patience and practice.\n\n",
            i
        ));
    }
    doc.push_str("## Section Two\n\n");
    doc.push_str("```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\n");
    for i in 30..60 {
        doc.push_str(&format!(
            "Another paragraph here, number {}. Sentences flow one after another. \
             Each word carefully chosen for clarity and impact.\n\n",
            i
        ));
    }
    doc
}

/// Simulate one frame of the rendering pipeline, matching the main loop's hot path.
fn simulate_frame(app: &mut zani::app::App, area: ratatui::layout::Rect) {
    use ratatui::buffer::Buffer as RatatuiBuffer;
    use ratatui::widgets::Widget;

    // 1. Compute visual lines (Rc clone on cache hit)
    let visual_lines = app.viewport.visual_lines(&app.editor.buffer);

    // 2. Ensure cursor visible
    app.viewport.ensure_cursor_visible(
        app.editor.cursor_line,
        app.editor.cursor_col,
        &visual_lines,
        area.height,
        &mut app.animations,
    );

    // 3. Update dimming (populates output buffers)
    let pb = app.editor.paragraph_bounds_cached();
    let sb = app.editor.sentence_bounds_cached();
    app.dimming.update(app.editor.buffer.len_lines(), pb, sb);

    // 4. Refresh render cache (reuses Vec capacity across frames)
    app.render_cache.refresh(&app.editor.buffer);

    // 5. Build and render WritingSurface (the heaviest part)
    let palette = app.effective_palette();
    let surface = zani::writing_surface::WritingSurface::new(&app.editor.buffer, &palette)
        .column_width(app.viewport.column_width)
        .scroll_offset(app.viewport.scroll_display.round() as usize)
        .cursor(app.editor.cursor_line, app.editor.cursor_col)
        .focus_mode(app.dimming.focus_mode)
        .sentence_bounds(sb)
        .sentence_fades(app.dimming.sentence_fade_snapshot())
        .color_profile(app.color_profile)
        .vertical_offset(app.viewport.typewriter_vertical_offset)
        .selection(app.editor.selection_range())
        .line_opacities(app.dimming.line_opacities())
        .precomputed_visual_lines(&visual_lines)
        .code_block_state(app.render_cache.code_block_state())
        .line_char_offsets(app.render_cache.line_char_offsets());

    let mut buf = RatatuiBuffer::empty(area);
    surface.render(area, &mut buf);
}

/// Single test to avoid global-counter races between parallel tests.
#[test]
fn measure_allocations_per_frame() {
    use zani::focus_mode::FocusMode;

    let doc = sample_document();
    let area = ratatui::layout::Rect::new(0, 0, 80, 40);
    let num_frames = 20;

    eprintln!("\n  === Allocation benchmark ({} logical lines, {}x{} terminal) ===\n",
        doc.lines().count(), area.width, area.height);

    // Test each focus mode
    let mut paragraph_per_frame = 0;
    for (mode_name, mode) in [
        ("Off", FocusMode::Off),
        ("Paragraph", FocusMode::Paragraph),
        ("Sentence", FocusMode::Sentence),
    ] {
        let mut app = zani::app::App::new();
        app.editor.buffer = zani::buffer::Buffer::from_text(&doc);
        app.dimming.focus_mode = mode;
        app.editor.cursor_line = 30; // Middle of document
        app.editor.cursor_col = 10;

        // Warm up: first frame populates all caches
        simulate_frame(&mut app, area);

        // Measure: steady-state frames (cursor hasn't moved, buffer unchanged)
        start_counting();
        for _ in 0..num_frames {
            simulate_frame(&mut app, area);
        }
        let total_allocs = stop_counting();
        let per_frame = total_allocs / num_frames;

        if mode == FocusMode::Paragraph {
            paragraph_per_frame = per_frame;
        }

        eprintln!(
            "  FocusMode::{:<12} {:>4} allocs/frame  ({} total over {} frames)",
            mode_name, per_frame, total_allocs, num_frames
        );
    }

    // Cold frame (first render) for comparison
    {
        let mut app = zani::app::App::new();
        app.editor.buffer = zani::buffer::Buffer::from_text(&doc);
        app.dimming.focus_mode = FocusMode::Paragraph;
        app.editor.cursor_line = 30;
        app.editor.cursor_col = 10;

        start_counting();
        simulate_frame(&mut app, area);
        let cold_allocs = stop_counting();

        eprintln!("  Cold frame (first render):  {} allocs", cold_allocs);
    }

    eprintln!();

    // Threshold assertion — tightened after Steps 1-5
    assert!(
        paragraph_per_frame < 100,
        "Steady-state allocations per frame ({paragraph_per_frame}) should be under 100 \
         (original baseline was ~440, current target is <100)"
    );
}
