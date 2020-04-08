use crate::map::Mapped;

const SIZE: usize = 40;

/// OAM table entry.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Entry {
    pub ypos: u8,
    pub xpos: u8,
    pub tile: u8,
    pub flags: u8,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            ypos: 0,
            xpos: 0,
            tile: 0,
            flags: 0,
        }
    }
}

pub struct Oam {
    entries: [Entry; SIZE],
    // TODO don't use an option
    visible: Option<Vec<Entry>>,
}

impl Default for Oam {
    fn default() -> Self {
        Self {
            entries: [Default::default(); SIZE],
            visible: Some(Vec::with_capacity(10)),
        }
    }
}

impl Oam {
    pub(crate) fn search(&mut self, ly: u8, height: u8) {
        let mut visible = self.visible.take().unwrap();
        visible.clear();
        let ly = ly as i16;
        let h = height as i16;
        for entry in 0..40 {
            let Entry { ypos, .. } = self.get(entry);
            // skip entry if it doesn't overlap with the current line
            // add to the array of visible sprites otherwise
            let y = *ypos as i16 - 16;
            if ly < y || ly >= y + h {
                continue;
            }
            visible.push(self.get(entry).clone());
            if visible.len() == 10 {
                break;
            }
        }
        //visible.sort_by_key(|e| e.xpos);
        visible.reverse();
        self.visible = Some(visible);
    }

    pub(crate) fn visible(&self) -> impl Iterator<Item = &Entry> {
        self.visible.iter().flat_map(|s| s)
    }

    /// Returns an iterator over the 40 OAM entries.
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    /// Returns an iterator over the 40 OAM entries as mutable.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entry> {
        self.entries.iter_mut()
    }

    /// Access an entry from the OAM table.
    ///
    /// # Panic
    /// Panics if `idx >= 40`
    pub fn get(&self, idx: usize) -> &Entry {
        &self.entries[idx]
    }

    /// Access an entry from the OAM table as mutable.
    ///
    /// # Panic
    /// Panics if `idx >= 40`
    pub fn get_mut(&mut self, idx: usize) -> &mut Entry {
        &mut self.entries[idx]
    }
}

impl Mapped for Oam {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xfe00..=0xfe9f => {
                let entry = (addr as usize - 0xfe00) / 4;
                let field = (addr - 0xfe00) % 4;
                match field {
                    0 => self.entries[entry].ypos,
                    1 => self.entries[entry].xpos,
                    2 => self.entries[entry].tile,
                    3 => self.entries[entry].flags,
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xfe00..=0xfe9f => {
                let entry = (addr as usize - 0xfe00) / 4;
                let field = (addr - 0xfe00) % 4;
                match field {
                    0 => self.entries[entry].ypos = data,
                    1 => self.entries[entry].xpos = data,
                    2 => self.entries[entry].tile = data,
                    3 => self.entries[entry].flags = data,
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }
}
