use gtk4::{EntryBuffer, Entry, prelude::EntryBufferExtManual, traits::{EditableExt, EntryExt}, Editable};

#[derive(Debug)]
enum EditEvent {
    InsertText {
        content : String,
        point   : i32,
    },
    DeleteText {
        content : String,
        start   : i32,
        end     : i32,
    }
}


impl EditEvent {
    fn apply_change(&self, buffer : EntryBuffer) -> Option<()> {
        match self {
            EditEvent::InsertText { content, point } 
            => Self::insert_text(buffer, *point, content.as_str()),
            EditEvent::DeleteText { start, end, .. } 
            => Self::delete_text(buffer, *start, *end),
        }
    }    

    fn unapply_change(&self, buffer : EntryBuffer) -> Option<()> {
        match self {
            EditEvent::InsertText { content, point } 
            => Self::delete_text(buffer, *point, *point + (content.chars().count() as i32)),
            EditEvent::DeleteText { content, start, .. } 
            => Self::insert_text(buffer, *start, content.as_str()),
        }
    }

    fn delete_text(buffer : EntryBuffer, start : i32, end : i32,) -> Option<()> {
        buffer.delete_text(
            start.try_into().ok()?, 
            Some((end - start).try_into().ok()?)
        );
        Some(())
    }

    fn insert_text(buffer : EntryBuffer, start : i32, text : &str) -> Option<()> {
        buffer.insert_text(
            start.try_into().ok()?, 
            text
        );
        Some(())
    }
} 

#[derive(Debug)]
struct Change {
    original_selection : (i32, i32),
    event : EditEvent,
}

#[derive(Debug)]
pub struct UndoStack {
    past   : Vec<Change>,
    future : Vec<Change>,
}

impl UndoStack {
    pub fn new() -> Self { 
        let past   = Vec::with_capacity(20);
        let future = Vec::with_capacity(5);
        Self { past, future, }
    }

    fn set_selection(entry : Entry, selection : (i32, i32)) {
        entry.select_region(selection.0, selection.1)
    }

    pub fn undo(&mut self, entry : Entry) -> bool {
        if let Some(mut change) = self.past.pop() {
            let original_selection = change.original_selection;
            change.original_selection = get_selection(&entry.delegate().unwrap());

            change.event.unapply_change(entry.buffer());
            Self::set_selection(entry, original_selection);
            self.future.push(change);
            true
        }
        else {
            false
        }
    }


    pub fn redo(&mut self, entry : Entry) -> bool {
        if let Some(mut change) = self.future.pop() {
            let original_selection = change.original_selection;
            change.original_selection = entry.selection_bounds().unwrap_or_else(||{
                let selection = entry.selection_bound();
                (selection, selection)
            });

            change.event.apply_change(entry.buffer());
            Self::set_selection(entry, original_selection);
            self.past.push(change);
            true
        }
        else {
            false
        }
    }

    pub fn insert_text(&mut self, new : &str, insertion_pt : i32, selection : (i32, i32)) {
        self.future.clear();
        self.past.push(Change { 
            event: EditEvent::InsertText {
                content: new.to_string(),
                point:   insertion_pt,
            },
            original_selection: selection, 
        });
    }

    pub fn delete_text(&mut self, deleted_chunk : &str, start : i32, end : i32, selection : (i32, i32)) {
        self.future.clear();
        self.past.push(Change { 
            event: EditEvent::DeleteText { 
                content: deleted_chunk.to_string(),
                start, end,
            },
            original_selection: selection, 
        });
    }

}

pub fn get_selection(entry: &Editable) -> (i32, i32) {
    entry.selection_bounds().unwrap_or_else(||{
        let selection = entry.selection_bound();
        (selection, selection)
    })
}





