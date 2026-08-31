use std::{sync::Arc, time::{Duration, Instant}};
use winit::{event::{ElementState, WindowEvent}, event_loop::EventLoop, keyboard::ModifiersState, window::Window};
use wgpu::*;
use keru_text::*;
use keru_text::parley::TextStyle;
use std::fmt::Write;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut Application { state: None }).unwrap();
}

struct State {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    text: Text,
    header: TextBoxHandle,
    first_frame_stats: TextBoxHandle,
    avg_stats: TextBoxHandle,
    frame_counter: TextBoxHandle,
    char_count: TextBoxHandle,
    help_text: TextBoxHandle,
    edit_boxes: Vec<TextEditHandle>,
    frame_count: u32,
    last_print_time: Instant,
    total_prepare_time: Duration,
    total_gpu_time: Duration,
    total_frame_time: Duration,
    first_gpu_time: Option<Duration>,
    first_prepare_time: Option<Duration>,
    first_frame_time: Option<Duration>,
    first_frame_stats_written: bool,
    scratch_string: String,
    last_frame_end: Instant,
    modifiers: ModifiersState,
    benchmark_mode: bool,
}

const BENCHMARK_MODE_DESC: &'static str = "Press Ctrl+S to toggle between realistic mode and benchmark mode.\nCurrent mode: benchmark. In this mode, we change the frame-counter text on every frame. This forces the renderer to reprepare all the text every frame. This is good for benchmarking prepare() itself, but for most applications, the other mode is more realistic.";
const REALISTIC_MODE_DESC: &'static str = "Press Ctrl+S to toggle between realistic mode and benchmark mode.\nCurrent mode: realistic. In this mode, the text only changes once per second. This means that even if the user is scrolling, we can get away with just moving around the quads that we prepared last frame instead of re-preparing everything. This helps a lot, even when the glyphs are already cached in the atlas.";


impl State {
    fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let device_desc = DeviceDescriptor {
            required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
            ..Default::default()
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&device_desc)).unwrap();
        let surface = instance.create_surface(window.clone()).unwrap();
        let mut surface_config = surface.get_default_config(&adapter, window.inner_size().width, window.inner_size().height).unwrap();
        // surface_config.present_mode = PresentMode::Fifo;
        surface_config.present_mode = PresentMode::Immediate;
        surface.configure(&device, &surface_config);

        let mut text = Text::new(&device, &queue, surface_config.format);

        let big_style = text.add_style(TextStyle {
            font_size: 24.0,
            brush: ColorBrush([255, 255, 255, 255]),
            ..Default::default()
        }, None);

        *text.get_default_text_style_mut() = TextStyle {
            font_size: 16.0,
            brush: ColorBrush([255, 255, 255, 255]),
            ..Default::default()
        };
        
        // Create header text box
        let header = text.add_text_box(
            "Warning: this is all outdated! The library is now much more incremental.
            This is an informal benchmark for the library and for the atlas renderer.\n\
            The first frame is very slow because of cpu-side glyph rasterization. In the following frames, the glyphs are cached in the atlases.\n\
            If you go really hard with ctrl+v on one of the text edit boxes, you'll quickly see the other limitation. Rendering can efficiently skip the text that's outside the clip area (or the screen), but the layouting and shaping can't.\n\n\
            \
            One funny issue is that Parley's Selection is always relative to the layout, not the raw text. This means that even when doing operations that could technically work on the raw text still, we still need to work on a fresh layout.\n\
            So, if you hold down ctrl+v and paste ten times in the time of a frame, we have to do ten layout rebuilds before rendering once. The subsequent pastes should replace the selection, so they need a fresh selection to replace, so they need a fresh layout. Ideally you'd be fine with just one rebuild per render.\n\
            This slows down the frame, giving you time to spam even more events before next one, and so on. When the whole program freezes for 20-30 seconds, it's because of this.\n\n\
            \
            Layouting a long text just once can still be quite slow, but this alone isn't enough to cause dramatic 30 second freezes. It can still cause the FPS to drop quite low, but from what I've seen, this effect is of the same order of magnitude as you see in browsers. Of course if text is split reasonably across multiple text boxes (one per UI element or one per paragraph) none of this is an issue.
             ",
            Some((10.0, 10.0)),
            (1850.0, 60.0),
            0.0
        );

        // Create stats display text boxes
        
        let stats_row_y = 270.0;
        let first_frame_stats = text.add_text_box(
            "",
            Some((110.0, stats_row_y)),
            (400.0, 100.0),
            0.0
        );

        let avg_stats = text.add_text_box(
            "Average: computing...".to_string(),
            Some((520.0, stats_row_y)),
            (400.0, 100.0),
            0.0
        );

        let char_count = text.add_text_box(
            "Total bytes of text:".to_string(),
            Some((880.0, stats_row_y)),
            (400.0, 100.0),
            0.0
        );
        text.get_text_box_mut(&char_count).set_style(&big_style);

        let frame_counter = text.add_text_box(
            "0",
            Some((1820.0, 20.0)),
            (90.0, 30.0),
            0.0
        );

        let help_text = text.add_text_box(
            REALISTIC_MODE_DESC,
            Some((1250.0, stats_row_y)),
            (600.0, 100.0),
            0.0
        );

        // Sample texts in different scripts
        let latin_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi";

        let arabic_text = "عندما يريد العالم أن يتكلم فهو يتحدث بلغة يونيكود. تسجل الآن لحضور المؤتمر الدولي العاشر ليونيكود الذي سيعقد في مارس المقبل بمدينة مايونتس في ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود حيث ستتم مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية والخطوط ورسم النصوص المترجمة والحوسبة متعددة اللغات. ليونيكود الذي سيعقد في مارس المقبل بمدينة مايونتس في ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود حيث ستتم مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية والخطوط ورسم النصوص المترجمة والحوسبة متعددة اللغات. ليونيكود الذي سيعقد في مارس المقبل بمدينة مايونتس في ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود حيث ستتم مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية والخطوط ورسم النصوص المترجمة والحوسبة متعددة اللغات. ليونيكود الذي سيعقد في مارس المقبل بمدينة مايونتس في ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود حيث ستتم مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية والخطوط ورسم النصوص المترجمة والحوسبة متعددة اللغات.";

        let chinese_text = "側経意責家方家閉討店暖育田庁載社転線宇。得君新術治温抗添代話考振投員殴大闘北裁。品間識部案代学凰処済準世一戸刻法分。悼測済諏計飯利安凶断理資沢同岩面文認革。内警格化再薬方久化体教御決数詭芸得筆代。指麦新設評掲聞測件索投権図囲写供表側経意責家方家閉討店暖育田庁載社転線宇。得君新術治温抗添代話考振投員殴大闘北裁。品間識部案代学凰処済準世一戸刻法分。悼測済諏計飯利安凶断理資沢同岩面文認革。内警格化再薬方久化体教御決数詭芸得筆代。指麦新設評掲聞測件索投権図囲写供表側経意責家方家閉討店暖育田庁載社転線宇。得君新術治温抗添代話考振投員殴大闘北裁。品間識部案代学凰処済準世一戸刻法分。悼測済諏計飯利安凶断理資沢同岩面文認革。内警格化再薬方久化体教御決数詭芸得筆代。指麦新設評掲聞測件索投権図囲写供表側経意責家方家閉討店暖育田庁載社転線宇。得君新術治温抗添代話考振投員殴大闘北裁。品間識部案代学凰処済準世一戸刻法分。悼測済諏計飯利安凶断理資沢同岩面文認革。内警格化再薬方久化体教御決数詭芸得筆代。指麦新設評掲聞測件索投権図囲写供表。";

        let japanese_text = "旅ロ京青利セムレ弱改フヨス波府かばぼ意送でぼ調掲察たス日西重ケアナ住橋ユムミク順待ふかんぼ人奨貯鏡すびそ。校文江方上温杯飛禁去克朝弘職掲器関権。阪社幸野岡愛権間投辞者庁対地更社号旅ロ京青利セムレ弱改フヨス波府かばぼ意送でぼ調掲察たス日西重ケアナ住橋ユムミク順待ふかんぼ人奨貯鏡すびそ。校文江方上温杯飛禁去克朝弘職掲器関権。阪社幸野岡愛権間投辞者庁対地更社号旅ロ京青利セムレ弱改フヨス波府かばぼ意送でぼ調掲察たス日西重ケアナ住橋ユムミク順待ふかんぼ人奨貯鏡すびそ。校文江方上温杯飛禁去克朝弘職掲器関権。阪社幸野岡愛権間投辞者庁対地更社号旅ロ京青利セムレ弱改フヨス波府かばぼ意送でぼ調掲察たス日西重ケアナ住橋ユムミク順待ふかんぼ人奨貯鏡すびそ。校文江方上温杯飛禁去克朝弘職掲器関権。阪社幸野岡愛権間投辞者庁対地更社号。";

        let cyrillic_text = "Лорем ипсум долор сит амет, пер цлита поссит ех, ат мунере фабулас петентиум сит. Иус цу цибо саперет сцрипсерит, нец виси муциус лабитур ид. Ет хис нонумес нолуиссе дигниссим. Ут но цонгуе цонсулату, ут усу путент алиенум индоцтум. При цу омнес епицуреиЛорем ипсум долор сит амет, пер цлита поссит ех, ат мунере фабулас петентиум сит. Иус цу цибо саперет сцрипсерит, нец виси муциус лабитур ид. Ет хис нонумес нолуиссе дигниссим. Ут но цонгуе цонсулату, ут усу путент алиенум индоцтум. При цу омнес епицуреиЛорем ипсум долор сит амет, пер цлита поссит ех, ат мунере фабулас петентиум сит. Иус цу цибо саперет сцрипсерит, нец виси муциус лабитур ид. Ет хис нонумес нолуиссе дигниссим. Ут но цонгуе цонсулату, ут усу путент алиенум индоцтум. При цу омнес епицуреиЛорем ипсум долор сит амет, пер цлита поссит ех, ат мунере фабулас петентиум сит. Иус цу цибо саперет сцрипсерит, нец виси муциус лабитур ид. Ет хис нонумес нолуиссе дигниссим. Ут но цонгуе цонсулату, ут усу путент алиенум индоцтум. При цу омнес епицуреи.";

        let greek_text = "Λορεμ ιπσθμ δολορ σιτ αμετ, μει ιδ νοvθμ φαβελλασ πετεντιθμ vελ νε, ατ νισλ σονετ οπορτερε εθμ. Αλιι δοcτθσ μει ιδ, νο αθτεμ αθδιρε ιντερεσσετ μελ, δοcενδι cομμθνε οπορτεατ τε cθμ. Πθρτο σαπιεντεμ ιν εαμ, σολθμ νθσqθαμ αδιπισcινγ ηασ νεΛορεμ ιπσθμ δολορ σιτ αμετ, μει ιδ νοvθμ φαβελλασ πετεντιθμ vελ νε, ατ νισλ σονετ οπορτερε εθμ. Αλιι δοcτθσ μει ιδ, νο αθτεμ αθδιρε ιντερεσσετ μελ, δοcενδι cομμθνε οπορτεατ τε cθμ. Πθρτο σαπιεντεμ ιν εαμ, σολθμ νθσqθαμ αδιπισcινγ ηασ νεΛορεμ ιπσθμ δολορ σιτ αμετ, μει ιδ νοvθμ φαβελλασ πετεντιθμ vελ νε, ατ νισλ σονετ οπορτερε εθμ. Αλιι δοcτθσ μει ιδ, νο αθτεμ αθδιρε ιντερεσσετ μελ, δοcενδι cομμθνε οπορτεατ τε cθμ. Πθρτο σαπιεντεμ ιν εαμ, σολθμ νθσqθαμ αδιπισcινγ ηασ νεΛορεμ ιπσθμ δολορ σιτ αμετ, μει ιδ νοvθμ φαβελλασ πετεντιθμ vελ νε, ατ νισλ σονετ οπορτερε εθμ. Αλιι δοcτθσ μει ιδ, νο αθτεμ αθδιρε ιντερεσσετ μελ, δοcενδι cομμθνε οπορτεατ τε cθμ. Πθρτο σαπιεντεμ ιν εαμ, σολθμ νθσqθαμ αδιπισcινγ ηασ νε.";

        // Create text boxes with different scripts
        let box_width = 600.0;
        let box_height = 310.0;
        let start_x = 10.0;
        let start_y = 400.0;
        let spacing_y = box_height as f64 + 10.0;

        let samples = vec![
            latin_text,
            greek_text,
            japanese_text,
            cyrillic_text,
            chinese_text,
            arabic_text,
        ];

        let mut edit_boxes = Vec::with_capacity(samples.len());
        for (col, text_content) in samples.iter().enumerate() {
            let row_offset = col / 3;
            let col_offset = col % 3;
            let x = start_x + col_offset as f64 * 640.0;
            let y = start_y + row_offset as f64 * spacing_y;

            let handle = text.add_text_edit(
                text_content.to_string(),
                Some((x, y)),
                (box_width, box_height),
                0.0
            );
            edit_boxes.push(handle);
        }

        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            text,
            first_frame_stats,
            avg_stats,
            header,
            frame_counter,
            char_count,
            help_text,
            frame_count: 0,
            last_print_time: Instant::now(),
            total_prepare_time: Duration::ZERO,
            total_gpu_time: Duration::ZERO,
            total_frame_time: Duration::ZERO,
            first_gpu_time: None,
            first_prepare_time: None,
            first_frame_time: None,
            first_frame_stats_written: false,
            scratch_string: String::with_capacity(64),
            last_frame_end: Instant::now(),
            edit_boxes,
            modifiers: ModifiersState::empty(),
            benchmark_mode: false,
        }
    }
}

struct Application { state: Option<State> }

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(event_loop.create_window(
                Window::default_attributes()
                    .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            ).unwrap());
            window.set_ime_allowed(true);
            self.state = Some(State::new(window));
        }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        state.text.handle_event(&event, &state.window);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                (state.surface_config.width, state.surface_config.height) = (size.width, size.height);
                state.surface.configure(&state.device, &state.surface_config);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && state.modifiers.control_key() {
                    if let Some(s) = event.text {
                        if s.as_str() == "s" {
                            state.benchmark_mode = !state.benchmark_mode;

                            state.scratch_string.clear();
                            let help_text = if state.benchmark_mode {
                                BENCHMARK_MODE_DESC
                            } else {
                                REALISTIC_MODE_DESC
                            };
                            state.text.get_text_box_mut(&state.help_text).set_text(&help_text);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {

                // Store first frame timings
                let prepare_time_for_first_frame = if state.first_prepare_time.is_none() {
                    Some(Instant::now())
                } else {
                    None
                };

                state.frame_count += 1;

                // In realtime mode, update frame counter and byte count every frame
                if state.benchmark_mode {
                    state.scratch_string.clear();
                    write!(&mut state.scratch_string, "Frame count:\n{}", state.frame_count).unwrap();
                    state.text.get_text_box_mut(&state.frame_counter).set_text(&state.scratch_string);

                    let mut char_count = 0;
                    char_count += state.text.get_text_box(&state.header).text().len();
                    char_count += state.text.get_text_box(&state.first_frame_stats).text().len();
                    char_count += state.text.get_text_box(&state.avg_stats).text().len();
                    char_count += state.text.get_text_box(&state.frame_counter).text().len();
                    char_count += state.text.get_text_box(&state.char_count).text().len();
                    char_count += state.text.get_text_box(&state.help_text).text().len();
                    for b in &state.edit_boxes {
                        char_count += state.text.get_text_edit(&b).raw_text().len();
                    }

                    state.scratch_string.clear();
                    write!(&mut state.scratch_string,
                        "Total bytes of text: {}", char_count
                    ).unwrap();
                    state.text.get_text_box_mut(&state.char_count).set_text(&state.scratch_string);
                }

                // Update first frame stats display when we have GPU time (only once)
                if state.first_gpu_time.is_some() && !state.first_frame_stats_written {
                    state.scratch_string.clear();
                    write!(&mut state.scratch_string,
                        "First frame:\nPrepare:    {:?}\nGPU Render: {:?}\nFrame:      {:?}",
                        state.first_prepare_time.unwrap(),
                        state.first_gpu_time.unwrap(),
                        state.first_frame_time.unwrap()
                    ).unwrap();
                    state.text.get_text_box_mut(&state.first_frame_stats).set_text(&state.scratch_string);
                    state.first_frame_stats_written = true;
                }

                // Update average statistics every second
                if state.last_print_time.elapsed() >= Duration::from_secs_f32(1.5) {
                    let avg_prepare = state.total_prepare_time / state.frame_count;
                    let avg_gpu = state.total_gpu_time / state.frame_count;
                    let avg_frame = state.total_frame_time / state.frame_count;
                    let fps = 1.0 / avg_frame.as_secs_f64();

                    state.scratch_string.clear();
                    write!(&mut state.scratch_string,
                        "Average (last 1 second):\nPrepare:    {:?}\nGPU Render: {:?}\nFrame:      {:?}\nFPS: {:.1}",
                        avg_prepare, avg_gpu, avg_frame, fps
                    ).unwrap();
                    state.text.get_text_box_mut(&state.avg_stats).set_text(&state.scratch_string);

                    // In one-second mode, update frame counter and byte count here
                    if !state.benchmark_mode {
                        state.scratch_string.clear();
                        write!(&mut state.scratch_string, "Frame count:\n{}", state.frame_count).unwrap();
                        state.text.get_text_box_mut(&state.frame_counter).set_text(&state.scratch_string);

                        let mut char_count = 0;
                        char_count += state.text.get_text_box(&state.header).text().len();
                        char_count += state.text.get_text_box(&state.first_frame_stats).text().len();
                        char_count += state.text.get_text_box(&state.avg_stats).text().len();
                        char_count += state.text.get_text_box(&state.frame_counter).text().len();
                        char_count += state.text.get_text_box(&state.char_count).text().len();
                        char_count += state.text.get_text_box(&state.help_text).text().len();
                        for b in &state.edit_boxes {
                            char_count += state.text.get_text_edit(&b).raw_text().len();
                        }

                        state.scratch_string.clear();
                        write!(&mut state.scratch_string,
                            "Total bytes of text: {}", char_count
                        ).unwrap();
                        state.text.get_text_box_mut(&state.char_count).set_text(&state.scratch_string);
                    }

                    // Reset counters
                    state.frame_count = 0;
                    state.last_print_time = Instant::now();
                    state.total_prepare_time = Duration::ZERO;
                    state.total_gpu_time = Duration::ZERO;
                    state.total_frame_time = Duration::ZERO;
                }

                // Render
                let prepare_start = Instant::now();
                state.text.prepare_all();
                let prepare_time = prepare_start.elapsed();
                state.total_prepare_time += prepare_time;

                if let Some(first_prepare_start) = prepare_time_for_first_frame {
                    state.first_prepare_time = Some(first_prepare_start.elapsed());
                }

                let surface_texture = match state.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    other => panic!("Failed to get current surface texture: {other:?}"),
                };
                let mut encoder = state.device.create_command_encoder(&CommandEncoderDescriptor::default());

                {
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &surface_texture.texture.create_view(&TextureViewDescriptor::default()),
                            resolve_target: None,
                            ops: Operations { load: LoadOp::Clear(Color::BLACK), store: StoreOp::Store },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });
                    state.text.render(&mut pass);
                }

                #[cfg(debug_assertions)]
                println!("{:?} {:?}", Instant::now(), state.text.render_stats());

                state.queue.submit(Some(encoder.finish()));

                state.queue.present(surface_texture);

                let frame_end = Instant::now();

                state.total_frame_time += frame_end.duration_since(state.last_frame_end);
                
                if state.first_frame_time.is_none() {
                    state.first_frame_time = Some(frame_end.duration_since(state.last_frame_end));
                }

                state.last_frame_end = frame_end;

                state.window.request_redraw();
            },
            _ => {}
        }
    }
}
