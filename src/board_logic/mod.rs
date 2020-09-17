use super::piece_logic::*;

#[cfg(test)]
mod tests;
pub struct ChessBoard {
    board: [[Option<Piece>; 8]; 8], //Hardcoded size bad? //flatten to 64, get helper function turn (x,y)-> pos
    white_king: (usize, usize),
    black_king: (usize, usize),
    passant_connection: Option<((usize, usize), (usize, usize))>,
    history: Vec<((usize, usize), (usize, usize))>,
    turn: (Color, usize),
}

impl ChessBoard {
    fn add_piece(&mut self, piece: Piece, position: (usize, usize)) {
        if self.board[position.1][position.0].is_none() {
            if piece.piece_type == PieceType::King {
                if piece.color == Color::White && self.white_king == (256, 256) {
                    self.white_king = position;
                    self.board[position.1][position.0] = Some(piece);
                } else if piece.color == Color::Black && self.black_king == (256, 256) {
                    self.black_king = position;
                    self.board[position.1][position.0] = Some(piece);
                } else {
                    panic!("Error adding King to table!")
                }
            } else {
                self.board[position.1][position.0] = Some(piece);
            }
        } else {
            panic!("Tried to add piece at non-empty space at {:?}", position)
        }
    }

    pub fn get_moves(
        &self,
        position: (usize, usize),
    ) -> Vec<((usize, usize), Option<SpecialMove>)> {
        let mut all_moves: Vec<((usize, usize), Option<SpecialMove>)> = Vec::new();
        for mov in self.regular_moves(position) {
            if !self.self_check(position, mov) {
                all_moves.push((mov, None));
            }
        }
        for mov in self.special_moves(position) {
            if mov.1 == SpecialMove::Pawn2Step {
                if !self.self_check(position, mov.0) {
                    all_moves.push((mov.0, Some(mov.1)));
                }
            } else {
                all_moves.push((mov.0, Some(mov.1)));
            }
        }

        all_moves
    }

    fn regular_moves(&self, position: (usize, usize)) -> Vec<(usize, usize)> {
        let mut results: Vec<(usize, usize)> = Vec::new();
        if let Some(piece) = self.ref_piece(position) {
            let mut legal_spaces: Vec<((usize, usize), Option<&Piece>)> = Vec::new();
            match piece.piece_type {
                PieceType::Pawn => {
                    let color_modifier = if piece.color == Color::White { 1 } else { -1 };
                    if let Ok(space) = self.check_move(position, (0, color_modifier)) {
                        if space.1.is_none() {
                            legal_spaces.push(space);
                        }
                    }
                    if let Ok(space) = self.check_move(position, (1, color_modifier)) {
                        if space.1.is_some() && space.1.unwrap().color != piece.color {
                            legal_spaces.push(space);
                        }
                    }
                    if let Ok(space) = self.check_move(position, (-1, color_modifier)) {
                        if space.1.is_some() && space.1.unwrap().color != piece.color {
                            legal_spaces.push(space);
                        }
                    }
                }
                _ => {
                    let (movement1, movement2) = piece.movement;
                    legal_spaces.extend(self.check_around(
                        position,
                        movement1,
                        piece.moves_continous,
                    ));
                    if let Some(movement) = movement2 {
                        legal_spaces.extend(self.check_around(
                            position,
                            movement,
                            piece.moves_continous,
                        ));
                    }
                }
            }
            for (move_to, space) in legal_spaces {
                if let Some(p) = space {
                    if p.color != piece.color {
                        results.push(move_to);
                    }
                } else {
                    results.push(move_to);
                }
            }
        }
        results.sort();
        results.dedup();
        results
    }

    fn special_moves(&self, position: (usize, usize)) -> Vec<((usize, usize), SpecialMove)> {
        let mut special_moves: Vec<((usize, usize), SpecialMove)> = Vec::new();
        if let Some(piece) = self.ref_piece(position) {
            if !piece.has_moved {
                if piece.piece_type == PieceType::Pawn {
                    let color_modifier = if piece.color == Color::White { 2 } else { -2 };
                    if let Ok(target_space) = self.check_move(position, (0, color_modifier)) {
                        if target_space.1.is_none() {
                            special_moves.push((target_space.0, SpecialMove::Pawn2Step));
                        }
                    }
                }
                if piece.piece_type == PieceType::King && !self.is_threatened(position, piece.color)
                {
                    let color_modifier = if piece.color == Color::White { 0 } else { 7 };
                    let mut spaces = self.check_continous(position, (1, 0));
                    if let Some(space) = spaces.pop() {
                        if let Some(rook) = space.1 {
                            if rook.piece_type == PieceType::Rook
                                && rook.color == piece.color
                                && !rook.has_moved
                            {
                                let mut can_castle = true;
                                for spots in spaces {
                                    if self.is_threatened(spots.0, piece.color) {
                                        can_castle = false;
                                    }
                                }
                                if can_castle {
                                    special_moves
                                        .push(((6, color_modifier), SpecialMove::CastlingRight));
                                }
                            }
                        }
                    }
                    let mut spaces = self.check_continous(position, (-1, 0));
                    if let Some(space) = spaces.pop() {
                        if let Some(rook) = space.1 {
                            if rook.piece_type == PieceType::Rook
                                && rook.color == piece.color
                                && !rook.has_moved
                            {
                                let mut can_castle = true;
                                for spots in spaces {
                                    if self.is_threatened(spots.0, piece.color) {
                                        can_castle = false;
                                    }
                                }
                                if can_castle {
                                    special_moves
                                        .push(((2, color_modifier), SpecialMove::CastlingLeft));
                                }
                            }
                        }
                    }
                }
            }
        }
        special_moves
    }

    fn check_around(
        &self,
        position: (usize, usize),
        moveset: (isize, isize),
        moves_continous: bool,
    ) -> Vec<((usize, usize), Option<&Piece>)> {
        let mut legal_spaces: Vec<((usize, usize), Option<&Piece>)> = Vec::new();
        let (move_x, move_y) = moveset;
        let directions = [
            (move_x, move_y),
            (-move_x, move_y),
            (move_x, -move_y),
            (-move_x, -move_y),
        ];
        for direction in directions.iter() {
            if moves_continous {
                legal_spaces.extend(self.check_continous(position, *direction));
            } else if let Ok(target_space) = self.check_move(position, *direction) {
                legal_spaces.push(target_space);
            }
        }
        legal_spaces
    }

    fn check_continous(
        &self,
        position: (usize, usize),
        direction: (isize, isize),
    ) -> Vec<((usize, usize), Option<&Piece>)> {
        let mut legal_spaces: Vec<((usize, usize), Option<&Piece>)> = Vec::new();
        if let Ok(target_space) = self.check_move(position, direction) {
            legal_spaces.push(target_space);
            if target_space.1.is_none() {
                legal_spaces.extend(self.check_continous(target_space.0, direction));
            }
        }
        legal_spaces
    }

    fn check_move(
        &self,
        position: (usize, usize),
        moves: (isize, isize),
    ) -> Result<((usize, usize), Option<&Piece>), String> {
        let new_x = position.0 as isize + moves.0;
        let new_y = position.1 as isize + moves.1;
        if new_x < 0 || new_x >= 8 || new_y < 0 || new_y >= 8 {
            return Err("not valid movement".to_string());
        }
        let new_pos = (new_x as usize, new_y as usize);
        let target_space = self.ref_piece(new_pos);
        if target_space.is_some() {
            Ok((new_pos, target_space))
        } else {
            Ok((new_pos, None))
        }
    }

    fn self_check(&self, move_from: (usize, usize), move_to: (usize, usize)) -> bool {
        let mut test = self.clone_chess();
        let piece: Piece = test.board[move_from.1][move_from.0].take().unwrap();
        test.board[move_to.1][move_to.0] = Some(piece);
        test.is_checked()
    }

    pub fn move_piece(
        &mut self,
        player_color: Color,
        move_from: (usize, usize),
        move_to: (usize, usize),
    ) -> Result<String, String> {
        if let Some(piece) = self.ref_piece(move_from) {
            if piece.color == player_color {
                if self.regular_moves(move_from).contains(&move_to) {
                    let piece: Piece = self.board[move_from.1][move_from.0].take().unwrap();
                    self.board[move_to.1][move_to.0] = Some(piece);
                    Ok(format!(
                        "{} {}",
                        super::to_notation(move_from).unwrap(),
                        super::to_notation(move_to).unwrap()
                    ))
                } else {
                    Err(format!(
                        "tried to do illegal move! Cannot move from {:?} to {:?}",
                        move_from, move_to
                    ))
                }
            } else {
                Err("That is not your piece!".to_string())
            }
        } else {
            Err("Tried to move empty space!".to_string())
        }
    }

    fn force_move(
        &mut self,
        piece_pos: (usize, usize),
        new_pos: (usize, usize),
    ) -> Result<String, String> {
        if let Some(piece) = self.board[piece_pos.1][piece_pos.0].take() {
            self.board[new_pos.1][new_pos.0] = Some(piece);
            Ok(format!("Moved from {:?} to {:?}", piece_pos, new_pos))
        } else {
            Err(format!("Can't force move, no piece at {:?}", piece_pos))
        }
    }

    fn is_threatened(&self, pos: (usize, usize), color: Color) -> bool {
        let mut direction = (0, 1);
        if self.is_threatened_by(pos, color, direction, true) {
            return true;
        }
        direction = (1, 1);
        if self.is_threatened_by(pos, color, direction, true) {
            return true;
        }
        direction = (1, 2);
        if self.is_threatened_by(pos, color, direction, false) {
            return true;
        }
        direction = (2, 1);
        if self.is_threatened_by(pos, color, direction, false) {
            return true;
        }
        false
    }

    pub fn is_threatened_by(
        &self,
        pos: (usize, usize),
        color: Color,
        moveset: (isize, isize),
        check_continous: bool,
    ) -> bool {
        for spaces in self.check_around(pos, moveset, check_continous) {
            if let (p_pos, Some(piece)) = spaces {
                if piece.color != color && self.regular_moves(p_pos).contains(&pos) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_checked(&self) -> bool {
        let king_pos = if self.turn.0 == Color::White {
            self.white_king
        } else {
            self.black_king
        };
        self.is_threatened(king_pos, self.turn.0)
    }

    pub fn is_checkmate(&self, color: Color) -> bool {
        let player_color = self.turn.0;
        let king_pos = if player_color == Color::White {
            self.white_king
        } else {
            self.black_king
        };
        if self.is_threatened(king_pos, color) {
            if self.regular_moves(king_pos).is_empty() {
                return false;
            }
            for y in 0..self.ref_board().len() {
                for x in 0..self.ref_board()[0].len() {
                    if let Some(piece) = self.ref_piece((x, y)) {
                        if piece.color == player_color && self.regular_moves((x, y)).is_empty() {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    pub fn standard_pieces(&mut self, color: Color) {
        let mut y = if color == Color::White { 1 } else { 6 };
        for x in 0..8 {
            self.add_piece(piece_make(color, PieceType::Pawn), (x, y));
        }
        y = if color == Color::White { 0 } else { 7 };
        self.add_piece(piece_make(color, PieceType::Rook), (0, y));
        self.add_piece(piece_make(color, PieceType::Knight), (1, y));
        self.add_piece(piece_make(color, PieceType::Bishop), (2, y));
        self.add_piece(piece_make(color, PieceType::Queen), (3, y));
        self.add_piece(piece_make(color, PieceType::King), (4, y));
        if color == Color::White {
            self.white_king = (4, y)
        } else {
            self.black_king = (4, y)
        }
        self.add_piece(piece_make(color, PieceType::Bishop), (5, y));
        self.add_piece(piece_make(color, PieceType::Knight), (6, y));
        self.add_piece(piece_make(color, PieceType::Rook), (7, y));
    }

    fn clone_chess(&self) -> ChessBoard {
        ChessBoard {
            board: self.board.clone(),
            white_king: self.white_king,
            black_king: self.black_king,
            passant_connection: self.passant_connection,
            history: self.history.clone(),
            turn: self.turn,
        }
    }

    pub fn ref_board(&self) -> &[[Option<Piece>; 8]; 8] {
        &self.board
    }

    pub fn ref_piece(&self, position: (usize, usize)) -> Option<&Piece> {
        self.board[position.1][position.0].as_ref()
    }
}

#[derive(PartialEq)]
pub enum SpecialMove {
    Pawn2Step,
    CastlingLeft,
    CastlingRight,
}

pub fn init_board() -> ChessBoard {
    ChessBoard {
        board: Default::default(),
        white_king: (256, 256),
        black_king: (256, 256),
        passant_connection: None,
        history: Vec::new(),
        turn: (Color::White, 1),
    }
}
