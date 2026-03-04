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
/// Takes a reusable ratatui buffer to avoid counting test-harness allocations.
fn simulate_frame(app: &mut zani::app::App, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
    use ratatui::widgets::Widget;

    // tick() performs all pre-draw state updates
    app.mark_needs_redraw();
    let out = app.tick(area.width, area.height).expect("should need redraw");

    // Build DrawContext — same extraction path as the real render loop
    let ctx = zani::ui::DrawContext::new(app, &out.visual_lines, out.sentence_bounds);

    // Build and render WritingSurface (the heaviest part)
    let surface = zani::writing_surface::WritingSurface::new(ctx.buffer, &ctx.effective_palette)
        .column_width(ctx.column_width)
        .scroll_offset(ctx.scroll_offset)
        .cursor(ctx.cursor_line, ctx.cursor_col)
        .focus_mode(ctx.focus_mode)
        .sentence_bounds(ctx.sentence_bounds)
        .sentence_fades(ctx.sentence_fades)
        .color_profile(ctx.color_profile)
        .vertical_offset(ctx.vertical_offset)
        .selection(ctx.selection)
        .line_opacities(ctx.line_opacities)
        .precomputed_visual_lines(ctx.visual_lines)
        .code_block_state(ctx.code_block_state)
        .line_char_offsets(ctx.line_char_offsets)
        .md_styles(ctx.md_styles)
        .precomputed_line_texts(ctx.line_texts)
        .precomputed_line_chars(ctx.line_chars);

    buf.reset();
    surface.render(area, buf);
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

    // Reusable ratatui buffer — allocated once, reused across all frames
    let mut render_buf = ratatui::buffer::Buffer::empty(area);

    // Test each focus mode, collecting per-frame allocation counts
    let mut off_per_frame = 0;
    let mut paragraph_per_frame = 0;
    let mut sentence_per_frame = 0;
    for (mode_name, mode) in [
        ("Off", FocusMode::Off),
        ("Paragraph", FocusMode::Paragraph),
        ("Sentence", FocusMode::Sentence),
    ] {
        let mut app = zani::app::App::new();
        app.set_buffer(zani::buffer::Buffer::from_text(&doc));
        app.set_focus_mode(mode);
        app.set_cursor(30, 10); // Middle of document

        // Warm up: first frame populates all caches
        simulate_frame(&mut app, area, &mut render_buf);

        // Measure: steady-state frames (cursor hasn't moved, buffer unchanged)
        start_counting();
        for _ in 0..num_frames {
            simulate_frame(&mut app, area, &mut render_buf);
        }
        let total_allocs = stop_counting();
        let per_frame = total_allocs / num_frames;

        match mode {
            FocusMode::Off => off_per_frame = per_frame,
            FocusMode::Paragraph => paragraph_per_frame = per_frame,
            FocusMode::Sentence => sentence_per_frame = per_frame,
        }

        eprintln!(
            "  FocusMode::{:<12} {:>4} allocs/frame  ({} total over {} frames)",
            mode_name, per_frame, total_allocs, num_frames
        );
    }

    // Cold frame (first render) for comparison
    {
        let mut app = zani::app::App::new();
        app.set_buffer(zani::buffer::Buffer::from_text(&doc));
        app.set_focus_mode(FocusMode::Paragraph);
        app.set_cursor(30, 10);

        start_counting();
        simulate_frame(&mut app, area, &mut render_buf);
        let cold_allocs = stop_counting();

        eprintln!("  Cold frame (first render):  {} allocs", cold_allocs);
    }

    eprintln!();

    // Threshold assertion — 0 allocs/frame for all focus modes
    assert!(
        off_per_frame == 0,
        "FocusMode::Off steady-state allocations ({off_per_frame}) should be 0"
    );
    assert!(
        paragraph_per_frame == 0,
        "FocusMode::Paragraph steady-state allocations ({paragraph_per_frame}) should be 0"
    );
    assert!(
        sentence_per_frame == 0,
        "FocusMode::Sentence steady-state allocations ({sentence_per_frame}) should be 0"
    );
}
