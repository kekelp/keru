use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    f: f32,
}

#[node_key] const K1: NodeKey;
#[node_key] const K2: NodeKey;
#[node_key] const K3: NodeKey;
#[node_key] const VSTACKKEY: NodeKey;
#[node_key] const WRAP: NodeKey;
#[node_key] const REDFILL: NodeKey;
#[node_key] const K7: NodeKey;
#[node_key] const K8: NodeKey;
#[node_key] const PANELKEY: NodeKey;
#[node_key] const K10: NodeKey;

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        // The constraints from the last relayout. Empty on the very first frame.
        let dot = ui.layout_dependencies_dot();
        if ! dot.is_empty() {
            std::fs::write("constraints.dot", &dot).unwrap();
        }


        // #[node_key] const FIT: NodeKey;
        // #[node_key] const FIXED: NodeKey;
        // #[node_key] const FILL: NodeKey;
        // #[node_key] const LABEL: NodeKey;

        // let fit = V_STACK
        //     .padding(10.0)
        //     .key(FIT)
        //     .size_x(Size::FitContent)
        //     .size_y(Size::FitContent)
        //     .color(Color::RED);

        // let fixed = PANEL
        //     .key(FIXED)
        //     .size_x(Size::Pixels(500.0))
        //     .color(Color::KERU_GREEN);
        
        // let fill = PANEL
        //     .key(FILL)
        //     .size_x(Size::Fill)
        //     .size_y(Size::Pixels(50.0))
        //     .color(Color::BLUE);

        // ui.add(fit).nest(|| {
        //     ui.add(fixed);
        //     ui.add(fill);
        // });




        // // // Cycle
        // #[node_key] const CYCLE: NodeKey;
        // #[node_key] const TALL: NodeKey;
        // #[node_key] const WIDE: NodeKey;
        // #[node_key] const LABEL: NodeKey;

        // ui.add(H_STACK).nest(|| {

        // });

        // ui.add(PANEL.key(CYCLE)).nest(|| {
        //     ui.add(PANEL.key(TALL)
        //         .size_x(Size::AspectRatio(0.5))
        //         .size_y(Size::Fill)
        //         .color(Color::KERU_BLUE));

        //     ui.add(PANEL.key(WIDE)
        //         .size_x(Size::Fill)
        //         .size_y(Size::AspectRatio(0.5))
        //         .color(Color::KERU_RED));
        // });



        // // Cycle with FitContent node depending on it
        // #[node_key] const CYCLE: NodeKey;
        // #[node_key] const TALL: NodeKey;
        // #[node_key] const WIDE: NodeKey;
        // #[node_key] const LABEL: NodeKey;
        // #[node_key] const VSTACK: NodeKey;
        // #[node_key] const PIXELS: NodeKey;

        // ui.add(V_STACK.key(VSTACK).padding(10.0).color(Color::KERU_GREEN)).nest(|| {

        //     ui.add(PANEL.key(CYCLE)).nest(|| {
        //         ui.add(PANEL.key(TALL)
        //             .size_x(Size::AspectRatio(0.5))
        //             .size_y(Size::Fill)
        //             .color(Color::KERU_BLUE));
    
        //         ui.add(PANEL.key(WIDE)
        //             .size_x(Size::Fill)
        //             .size_y(Size::AspectRatio(0.5))
        //             .color(Color::KERU_RED));
        //     });
    
        //     // This is too easy, even without the deferred queue, the fallback to min size would solve it correctly.
        //     // ui.add(PANEL.key(PIXELS).size(Size::Pixels(100.0), Size::Pixels(50.0)));

        //     // to actually see the difference, we need something that actually relies on the new algorithm, like the classic giraffe.
        //     #[node_key] const HSTACK: NodeKey;
        //     #[node_key] const GIRAFFE: NodeKey;
        //     #[node_key] const PARAGRAPH: NodeKey;
        //     #[node_key] const VSTACK: NodeKey;

        //     let giraffe = PANEL
        //         .size_y(Size::Fill)
        //         .size_x(Size::AspectRatio(0.5))
        //         .linear_gradient(LinearGradient { color_start: Color::KERU_BLUE, color_end: Color::KERU_RED, angle_deg: 0.0 })
        //         .key(GIRAFFE);
        //     let paragraph = TEXT_PARAGRAPH
        //         .size_x(Size::Fill)
        //         .text_size(14.0)
        //         .text("Wrapping paragraph with a lot of text that overflows. Wrapping paragraph with a lot of text that overflows. Wrapping paragraph with a lot of text that overflows.");
        //     let h_stack = H_STACK.key(HSTACK).color(Color::KERU_PINK).padding(10.0);
            
        //     ui.add(h_stack).nest(|| {
        //         ui.add(giraffe);
        //         ui.add(V_STACK.key(VSTACK)).nest(|| {
        //             ui.add(TEXT.text("Single line"));
        //             ui.add(paragraph);
        //         })
        //     });

        // });


        // let big_button = BUTTON
        //     .size_symm(Size::Fill)
        //     .color(Color::RED.with_alpha(0.5))
        //     .static_text("Text text")
        //     .stack(Axis::Y, Arrange::Center, 10.0)
        //     ;

        // ui.add(PANEL.size_y(Size::Fill).size_x(Size::FitContent)).nest(|| {
        //     ui.add(big_button)
        // });


        // self.add(PANEL.size_y(Size::Fill).size_x(Size::FitContent)).nest(|| {
        //     self.add(big_button).nest(|| {
        //         self.spacer();
        //         self.add(nested_button_1);
        //         self.spacer();
        //         self.add(nested_button_2);
        //         self.spacer();
        //     });
        // });

        // Stale fill level: the stack's fill level is computed lazily by the first Fill child that gets solved, and it reads its siblings through l2_size_or_guess. SQUARE's X is only knowable at the end of a 5-hop chain (row.Y -> SQUARE.Y -> INNER.Y -> INNER.X -> SQUARE.X), while FILL's X is one hop off row.X, so FILL is solved first and bakes in a guess of SQUARE's width.
        // Row is 600x400. INNER is Fill on Y, so it comes out 400 tall, and AspectRatio(1.0) makes it 400 wide, so SQUARE should be 400 wide and FILL should get the remaining 200.
        // If the level is stale, SQUARE is guessed at its base-pass value (min 0, no preferred, because a Fill child contributes nothing to a preferred size), the budget looks like the whole 600, and FILL comes out 600 wide, overflowing the row by 400.
        #[node_key] const ROW: NodeKey;
        #[node_key] const SQUARE: NodeKey;
        #[node_key] const FIT1: NodeKey;
        #[node_key] const INNER: NodeKey;
        #[node_key] const FILL: NodeKey;

        let row = H_STACK
            .key(ROW)
            .size_x(Size::Pixels(600.0))
            .size_y(Size::Pixels(400.0))
            .padding(0.0)
            .stack(Axis::X, Arrange::Start, 0.0)
            .color(Color::KERU_PINK);

        // FIT1 is the whole trick, and one layer of it is enough: the base pass settles Fill and AspectRatio eagerly, so INNER costs no queue rounds at all and SQUARE.X would beat FILL.X without it, but a FitContent is never settled eagerly, so each layer costs a real round. INNER still leaves both of them with no preferred size, so the guess stays bad while the arrival gets late. Take FIT1 out and FILL correctly comes out at 200.
        let fit = PANEL.size_x(Size::FitContent).size_y(Size::Fill).padding(0.0);

        ui.add(row).nest(|| {
            ui.add(fit.key(SQUARE).color(Color::KERU_GREEN)).nest(|| {
                ui.add(fit.key(FIT1)).nest(|| {
                    ui.add(PANEL.key(INNER).size_y(Size::Fill).size_x(Size::AspectRatio(1.0)).color(Color::KERU_BLUE));
                });
            });

            ui.add(PANEL.key(FILL).size_x(Size::Fill).size_y(Size::Fill).color(Color::KERU_RED));
        });

        // Unclamped Frac base. This one has nothing to do with ordering: l2_stack_child_base hands back a Frac's share as available * f without ever clamping it, but the child's own Final does go through l2_clamp and gets pulled up to its min. So the budget is reduced by the share the Frac asked for rather than by the size it actually takes, and the Fill child spends room that isn't there.
        // FRACC asks for 0.1 of 600 = 60, but its min drags it to 400. The budget should be 600 - 400 = 200, so FILL2 should be 200. If the base is unclamped the budget looks like 600 - 60 = 540, and FILL2 comes out 540 next to a 400-wide FRACC, overflowing the row by 340.
        #[node_key] const FRACROW: NodeKey;
        #[node_key] const FRACC: NodeKey;
        #[node_key] const FILL2: NodeKey;

        let frac_row = H_STACK
            .key(FRACROW)
            .size_x(Size::Pixels(600.0))
            .size_y(Size::Pixels(400.0))
            .padding(0.0)
            .stack(Axis::X, Arrange::Start, 0.0)
            .color(Color::KERU_PINK);

        ui.add(frac_row).nest(|| {
            ui.add(PANEL.key(FRACC).size_x(Size::Frac(0.1)).min_size_x(Size::Pixels(400.0)).size_y(Size::Fill).color(Color::KERU_GREEN));
            ui.add(PANEL.key(FILL2).size_x(Size::Fill).size_y(Size::Fill).color(Color::KERU_RED));
        });

        // // Giraffe
        // #[node_key] const HSTACK: NodeKey;
        // #[node_key] const GIRAFFE: NodeKey;
        // #[node_key] const PARAGRAPH: NodeKey;
        // #[node_key] const VSTACK: NodeKey;

        // let giraffe = PANEL
        //     .size_y(Size::Fill)
        //     .size_x(Size::AspectRatio(0.5))
        //     .linear_gradient(LinearGradient { color_start: Color::KERU_BLUE, color_end: Color::KERU_RED, angle_deg: 0.0 })
        //     .key(GIRAFFE);

        // let fake_paragraph = PANEL
        //     .size_y(Size::AspectRatio(1.0))
        //     .size_x(Size::Fill)
        //     .linear_gradient(LinearGradient { color_start: Color::KERU_BLUE, color_end: Color::KERU_RED, angle_deg: 90.0 })
        //     .key(PARAGRAPH);

        // let paragraph = TEXT_PARAGRAPH
        //     .size_x(Size::Fill)
        //     .text_size(14.0)
        //     .text("Wrapping paragraph with a lot of text that overflows. Wrapping paragraph with a lot of text that overflows. Wrapping paragraph with a lot of text that overflows.");

        // let h_stack = H_STACK.key(HSTACK).color(Color::KERU_PINK).padding(10.0);
        
        // ui.add(h_stack).nest(|| {

        //     ui.add(giraffe);

        //     ui.add(V_STACK.key(VSTACK)).nest(|| {
        //         ui.add(TEXT.text("Single line"));

        //         ui.add(paragraph);
        //         // ui.add(fake_paragraph);

        //     })
        // });

    

        // #[node_key] const HSTACK: NodeKey;
        // #[node_key] const GIRAFFE: NodeKey;
        // #[node_key] const PARAGRAPH: NodeKey;
        // #[node_key] const VSTACK: NodeKey;

        // let v_stack = V_STACK.key(HSTACK).color(Color::KERU_RED).padding(10.0);
        // let panel = PANEL.size_x(Size::Pixels(70.0));

        // ui.add(v_stack).nest(|| {
        //     ui.add(panel.size_y(Size::Pixels(200.0)).color(Color::BLUE));
        //     // ui.add(panel.size_y(Size::Fill).color(Color::BLUE));
        //     ui.add(panel.size_y(Size::Frac(0.3333)).color(Color::BLUE));
        //     ui.add(panel.size_y(Size::Frac(0.3333)).color(Color::BLUE));
        // });
    
    
        
        // ui.add(V_STACK.key(K1)).nest(|| {
        //     ui.add(PANEL.key(K2).size_symm(Size::Fill).color(Color::BLUE)).nest(|| {
        //         ui.add(PANEL.key(K3).size_symm(Size::Fill).color(Color::RED)).nest(|| {
        //             ui.add(PANEL.key(K4).size_symm(Size::Fill).color(Color::GREEN)).nest(|| {
        //                 ui.add(PANEL.key(K5).size_symm(Size::Fill).color(Color::KERU_PINK)).nest(|| {
        //                     ui.add(TEXT.text("Feed"));
        //                 })
        //             });
        //         });
        //     });
        // });


        // #[node_key] const VSTACK: NodeKey;
        // #[node_key] const PIXELS: NodeKey;
        // #[node_key] const PIXELS2: NodeKey;
        // #[node_key] const FILL: NodeKey;

        // ui.add(V_STACK.key(VSTACK).color(Color::BLUE)).nest(|| {
        //     ui.add(PANEL.key(PIXELS).size_y(Size::Pixels(50.0)).size_x(Size::Pixels(200.0)));

        //     ui.add(PANEL.key(FILL).size_y(Size::Pixels(50.0)).size_x(Size::Fill)).nest(|| {
        //         ui.add(PANEL.key(PIXELS).size_y(Size::Fill).size_x(Size::Pixels(400.0)).color(Color::KERU_GREEN));
        //     });

        // });


        // #[node_key] const VSTACK: NodeKey;
        // #[node_key] const PIXELS: NodeKey;
        // #[node_key] const PIXELS2: NodeKey;
        // #[node_key] const FILL: NodeKey;
        // #[node_key] const LABEL: NodeKey;

        // ui.add(V_STACK.key(VSTACK)).nest(|| {                       // FitContent X
        //     ui.add(PANEL.key(PIXELS).size_x(Size::Pixels(200.0)));
        //     ui.add(PANEL.key(FILL).size_x(Size::FitContent)).nest(|| {
        //         ui.add(TEXT.key(LABEL).text("some fairly long label"));  // FitContent X
        //     });
        // });
            

        // #[node_key] const COL: NodeKey;
        // #[node_key] const HEADER: NodeKey;
        // #[node_key] const HALF: NodeKey;
        // #[node_key] const BIG: NodeKey;
      
        // let a = PANEL.key(HEADER).size_x(Size::Pixels(50.0)).size_y(Size::Pixels(100.0));

        // ui.add(V_STACK.key(COL).size_y(Size::FitContent)).nest(|| {
        //     ui.add(a);
        //     ui.add(PANEL.key(HALF).size_y(Size::Frac(0.5)).color(Color::KERU_BLUE)).nest(|| {
        //         ui.add(PANEL.key(BIG).size_y(Size::Pixels(400.0)).color(Color::KERU_GREEN));
        //     });
        // });
      

        // #[node_key] const ROW: NodeKey;
        // #[node_key] const FIXED: NodeKey;
        // #[node_key] const FILL_A: NodeKey;
        // #[node_key] const FILL_B: NodeKey;
        // #[node_key] const LABEL: NodeKey;
      
        // ui.add(H_STACK.key(ROW).size_x(Size::Fill).size_y(Size::FitContent)).nest(|| {
        //     ui.add(PANEL.key(FIXED).size_x(Size::Pixels(100.0)).size_y(Size::Pixels(20.0)));
        //     ui.add(PANEL.key(FILL_A).size_x(Size::Fill).size_y(Size::Fill)).nest(|| {
        //         ui.add(TEXT.key(LABEL).text("a label long enough to need two lines"));
        //     });
        //     ui.add(PANEL.key(FILL_B).size_x(Size::Fill).size_y(Size::Fill));
        // });
      

    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
