use tui_input::{Input, InputRequest};

use crate::app::Action;

pub fn update(input: &mut Input, action: Action) -> bool {
    let request = match action {
        Action::Insert(character) if !character.is_control() => InputRequest::InsertChar(character),
        Action::DeleteBefore => InputRequest::DeletePrevChar,
        Action::DeleteAfter => InputRequest::DeleteNextChar,
        Action::DeleteWordBefore => InputRequest::DeletePrevWord,
        Action::DeleteWordAfter => InputRequest::DeleteNextWord,
        Action::CursorPrevious => InputRequest::GoToPrevChar,
        Action::CursorNext => InputRequest::GoToNextChar,
        Action::CursorWordPrevious => InputRequest::GoToPrevWord,
        Action::CursorWordNext => InputRequest::GoToNextWord,
        Action::CursorFirst => InputRequest::GoToStart,
        Action::CursorLast => InputRequest::GoToEnd,
        Action::ClearInput => InputRequest::DeleteLine,
        _ => return false,
    };
    input.handle(request).is_some_and(|changed| changed.value)
}
