#![allow(unused)]

use maltebl_chess::chess_game::*;
use maltebl_chess::*;

use druid::widget::*;
use druid::WidgetExt;
use druid::*;

use druid::piet::Color;

use std::rc::Rc;
use std::sync::Arc;

const NUM_ROWS: i32 = 8;
const NUM_COLS: i32 = 8;

fn main() -> Result<(), PlatformError> {
    let data = AppState::new();
    let window = WindowDesc::new(main_ui)
        .title(|data: &AppState, _env: &Env| format!("Chess {:?} {:?}", data.origin, data.message))
        .resizable(false)
        .window_size((300.0, 300.0));

    // let tile = &chess.chess_board.board[0][0];
    // println!("{}", tile.clone().unwrap());

    AppLauncher::with_window(window).launch(data);
    Ok(())
}

fn playground() -> Result<String, String> {
    let mut game = init_standard_chess();
    let cpos = (0 as usize, 1 as usize);
    let tpos = (0 as usize, 2 as usize);
    let command = format!("{} {}", to_notation(cpos)?, to_notation(tpos)?);
    let msg = game.move_piece(command.clone())?;
    // game.print_board();
    let piece_char = game.get_piece((0, 0)).as_ref().map(|x| format!("{}", x)).unwrap_or(" ".to_owned());
    println!("{}, {}, {}", piece_char, msg, command);
    Ok("".to_owned())
}

#[derive(Data, Copy, Clone, Debug, PartialEq)]
struct Position(i32, i32);
#[derive(Data, Clone)]
struct AppState {
    game: Arc<ChessGame>,
    origin: Option<Position>,
    message: Option<String>,
}

impl AppState {
    fn new() -> Self {
        let game = init_standard_chess();

        Self {
            game: Arc::new(game),
            origin: None,
            message: None,
        }
    }
}

fn main_ui() -> impl Widget<AppState> {
    let make_tile = |pos: Position| {
        let _b = Button::dynamic(move |data: &AppState, _| {
            let cpos = (pos.0 as usize, pos.1 as usize);
            data.game
                .get_piece(cpos)
                .as_ref()
                .map(|x| format!("{}", x))
                .unwrap_or(" ".to_owned())
        })
        .on_click(move |ctx, data, env| {
            match data.origin {
                None => {
                    let cpos = (pos.0 as usize, pos.1 as usize);
                    data.origin = Some(pos);
                    // data.origin_available_moves = Some(Arc::new(data.game.chess_board.get_moves(cpos)));
                }
                Some(prev) => {
                    let is_target = prev != pos;
                    if is_target {
                        // TODO: emit action: target: pos
                        let cpos = (prev.0 as usize, prev.1 as usize);
                        let tpos = (pos.0 as usize, pos.1 as usize);

                        let mut doit = || -> Result<String, String> {
                            let command = format!("{} {}", to_notation(cpos)?, to_notation(tpos)?);
                            // Arc::make_mut(&mut data.game).expect("couldn't get mut game ref").move_piece(command)
                            Ok("".to_owned())
                        };

                        let txt = match doit() {
                            Err(inner) => format!("Error: {}", inner),
                            Ok(inner) => format!("{}", inner),
                        };

                        data.message = if txt.len() > 0 { Some(txt) } else { None };
                    };
                    data.origin = None;
                }
            }
        })
        .env_scope(move |env, data: &AppState| {
            let is_selected = data.origin.filter(|a| *a == pos).is_some();
            let checkerboard = pos.0 % 2 == pos.1 % 2;
            let col = if checkerboard { 0x000000FF } else { 0x777777FF };
            let col = col + if is_selected { 0x33333300 } else { 0 };
            let col = Color::from_rgba32_u32(col);

            env.set(theme::BUTTON_DARK, col.clone());
            env.set(theme::BUTTON_LIGHT, col);
            env.set(theme::BUTTON_BORDER_WIDTH, 0.0);
            env.set(theme::BUTTON_BORDER_RADIUS, 0.0);
        });

        Tile::new(pos)
    };

    let make_row = |y: i32| {
        (0..NUM_COLS).fold(Flex::row(), |col, x| {
            col.with_flex_child(make_tile(Position(x, y)), 1.0)
        })
    };

    let make_rows = || {
        (0..NUM_ROWS).fold(Flex::column(), |row, y| {
            row.with_flex_child(make_row(y), 1.0)
        })
    };

    // let label = Label::dynamic(|data: &AppState, _| format!("Origin: {:?}; {:?}", data.origin, data.message));
    let board = make_rows()
        .center()
        .padding(15.0)
        .background(Color::from_rgba32_u32(0x332205FF));

    // let board = Flex::column().with_flex_child(board, 1.0).padding((10.0, 0.0, 10.0, 0.0));

    Flex::column()
        // .with_child(label.padding(5.).center())
        .with_flex_child(board, 1.0)
}

struct Tile {
    position: Position,
}

impl Tile {
    fn new(position: Position) -> Self {
        Self { position }
    }
}

struct ColorUtil;
impl ColorUtil {
    pub fn hsl(h: f64, s: f64, l: f64) -> Color {
        Self::rbg8t(Self::hsl_to_rgb(h, s, l))
    }
    pub const fn rbg8t((r, g, b): (u8, u8, u8)) -> Color {
        Color::rgb8(r, g, b)
    }
    // https://pauljmiller.com/posts/druid-widget-tutorial.html
    fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
        let mut t = t;
        if t < 0. {
            t += 1.
        }
        if t > 1. {
            t -= 1.
        };
        if t < 1. / 6. {
            return p + (q - p) * 6. * t;
        }
        if t < 1. / 2. {
            return q;
        }
        if t < 2. / 3. {
            return p + (q - p) * (2. / 3. - t) * 6.;
        }
        return p;
    }

    fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
        let r;
        let g;
        let b;

        if s == 0.0 {
            r = l;
            g = l;
            b = l; // achromatic
        } else {
            let q = if l < 0.5 { l * (1. + s) } else { l + s - l * s };

            let p = 2. * l - q;
            r = Self::hue_to_rgb(p, q, h + 1. / 3.);
            g = Self::hue_to_rgb(p, q, h);
            b = Self::hue_to_rgb(p, q, h - 1. / 3.);
        }

        return (
            (r * 255.).round() as u8,
            (g * 255.).round() as u8,
            (b * 255.).round() as u8,
        );
    }
}

impl<T> Widget<T> for Tile {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {}
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {}
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {}
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.max()
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let pos = self.position;
        // let is_selected = data.origin.filter(|a| *a==pos).is_some();
        let checkerboard = pos.0 % 2 == pos.1 % 2;

        let bounds = ctx.size().to_rect();
        let colo = ColorUtil::hsl(0.1, 0.2, if checkerboard { 0.1 } else { 0.5 });

        ctx.fill(bounds, &colo);
    }
}
