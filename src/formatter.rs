use crate::{Debug, FmtOpts, Result, Write};

pub struct Formatter<W, O> {
    pub(crate) buf: W,
    pub(crate) opts: O,
}

impl<W: Write, O: FmtOpts> core::fmt::Write for Formatter<W, O> {
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.buf.write_char(c).map_err(|_| std::fmt::Error)
    }
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s).map_err(|_| std::fmt::Error)
    }
}

impl<W> Formatter<W, ()> {
    pub fn new(buf: W) -> Self {
        Self { buf, opts: () }
    }
}

impl<W: Write, O: FmtOpts> Formatter<W, O> {
    pub fn write_char(&mut self, char: char) -> Result {
        self.buf.write_char(char)
    }

    pub fn write_str(&mut self, str: &str) -> Result {
        self.buf.write_str(str)
    }

    pub fn debug_list<'b>(&'b mut self) -> DebugList<'b, W, O> {
        debug_list_new(self)
    }

    pub fn debug_set<'b>(&'b mut self) -> DebugSet<'b, W, O> {
        debug_set_new(self)
    }

    pub fn debug_map<'b>(&'b mut self) -> DebugMap<'b, W, O> {
        debug_map_new(self)
    }

    fn wrap_buf<'buf, F, Wrap>(&'buf mut self, wrap: F) -> Formatter<Wrap, O>
    where
        F: FnOnce(&'buf mut W) -> Wrap,
    {
        Formatter {
            buf: wrap(&mut self.buf),
            opts: self.opts,
        }
    }
}

impl<W, O: FmtOpts> Formatter<W, O> {
    pub(crate) fn wrap_with<ONew: FmtOpts>(
        &mut self,
        opts: &ONew,
    ) -> Formatter<&mut W, ONew::ReplaceInnermost<O>> {
        Formatter {
            buf: &mut self.buf,
            opts: opts.override_other(self.opts),
        }
    }
}

/////////////////////////////////////////
////////////// BUILDERS /////////////////
/////////////////////////////////////////
// adapted from `core`
use crate as fmt;

struct PadAdapter<'state, 'buf, W> {
    buf: &'buf mut W,
    state: &'state mut PadAdapterState,
}

struct PadAdapterState {
    on_newline: bool,
}

impl Default for PadAdapterState {
    fn default() -> Self {
        PadAdapterState { on_newline: true }
    }
}

impl<'state, 'buf, W> PadAdapter<'state, 'buf, W>
where
    W: Write + 'buf,
{
    fn wrap<'slot, 'fmt, O: FmtOpts>(
        fmt: &'fmt mut Formatter<W, O>,
        slot: &'slot mut Option<Self>,
        state: &'state mut PadAdapterState,
    ) -> fmt::Formatter<&'slot mut PadAdapter<'state, 'buf, W>, O>
    where
        'fmt: 'buf + 'slot,
    {
        fmt.wrap_buf(move |buf| slot.insert(PadAdapter { buf, state }))
    }
}

impl<W: Write> fmt::Write for PadAdapter<'_, '_, W> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            if self.state.on_newline {
                self.buf.write_str("    ")?;
            }

            let split = match s.find('\n') {
                Some(pos) => {
                    self.state.on_newline = true;
                    pos + 1
                }
                None => {
                    self.state.on_newline = false;
                    s.len()
                }
            };
            self.buf.write_str(&s[..split])?;
            s = &s[split..];
        }

        Ok(())
    }
}

#[must_use = "must eventually call `finish()` on Debug builders"]
pub struct DebugStruct<'a, W, O> {
    fmt: &'a mut fmt::Formatter<W, O>,
    result: fmt::Result,
    has_fields: bool,
}

pub(super) fn debug_struct_new<'a, W: Write, O: FmtOpts>(
    fmt: &'a mut fmt::Formatter<W, O>,
    name: &str,
) -> DebugStruct<'a, W, O> {
    let result = fmt.write_str(name);
    DebugStruct {
        fmt,
        result,
        has_fields: false,
    }
}

impl<'a, W: Write, O: FmtOpts> DebugStruct<'a, W, O> {
    pub fn field(&mut self, name: &str, value: &impl Debug) -> &mut Self {
        self.result = self.result.and_then(|_| {
            if self.is_pretty() {
                if !self.has_fields {
                    self.fmt.write_str(" {\n")?;
                }
                let mut slot = None;
                let mut state = Default::default();
                let mut writer = PadAdapter::wrap(self.fmt, &mut slot, &mut state);
                writer.write_str(name)?;
                writer.write_str(": ")?;
                value.fmt(&mut writer)?;
                writer.write_str(",\n")
            } else {
                let prefix = if self.has_fields { ", " } else { " { " };
                self.fmt.write_str(prefix)?;
                self.fmt.write_str(name)?;
                self.fmt.write_str(": ")?;
                value.fmt(self.fmt)
            }
        });

        self.has_fields = true;
        self
    }

    pub fn finish_non_exhaustive(&mut self) -> fmt::Result {
        self.result = self.result.and_then(|_| {
            if self.has_fields {
                if self.is_pretty() {
                    let mut slot = None;
                    let mut state = Default::default();
                    let mut writer = PadAdapter::wrap(self.fmt, &mut slot, &mut state);
                    writer.write_str("..\n")?;
                    self.fmt.write_str("}")
                } else {
                    self.fmt.write_str(", .. }")
                }
            } else {
                self.fmt.write_str(" { .. }")
            }
        });
        self.result
    }

    pub fn finish(&mut self) -> fmt::Result {
        if self.has_fields {
            self.result = self.result.and_then(|_| {
                if self.is_pretty() {
                    self.fmt.write_str("}")
                } else {
                    self.fmt.write_str(" }")
                }
            });
        }
        self.result
    }

    fn is_pretty(&self) -> bool {
        self.fmt.alternate()
    }
}

#[must_use = "must eventually call `finish()` on Debug builders"]
#[allow(missing_debug_implementations)]
pub struct DebugTuple<'a, W, O> {
    fmt: &'a mut fmt::Formatter<W, O>,
    result: fmt::Result,
    fields: usize,
    empty_name: bool,
}

pub(super) fn debug_tuple_new<'a, W: Write, O: FmtOpts>(
    fmt: &'a mut fmt::Formatter<W, O>,
    name: &str,
) -> DebugTuple<'a, W, O> {
    let result = fmt.write_str(name);
    DebugTuple {
        fmt,
        result,
        fields: 0,
        empty_name: name.is_empty(),
    }
}

impl<'a, W: Write, O: FmtOpts> DebugTuple<'a, W, O> {
    pub fn field(&mut self, value: &impl Debug) -> &mut Self {
        self.result = self.result.and_then(|_| {
            if self.is_pretty() {
                if self.fields == 0 {
                    self.fmt.write_str("(\n")?;
                }
                let mut slot = None;
                let mut state = Default::default();
                let mut writer = PadAdapter::wrap(self.fmt, &mut slot, &mut state);
                value.fmt(&mut writer)?;
                writer.write_str(",\n")
            } else {
                let prefix = if self.fields == 0 { "(" } else { ", " };
                self.fmt.write_str(prefix)?;
                value.fmt(self.fmt)
            }
        });

        self.fields += 1;
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        if self.fields > 0 {
            self.result = self.result.and_then(|_| {
                if self.fields == 1 && self.empty_name && !self.is_pretty() {
                    self.fmt.write_str(",")?;
                }
                self.fmt.write_str(")")
            });
        }
        self.result
    }

    fn is_pretty(&self) -> bool {
        self.fmt.alternate()
    }
}

struct DebugInner<'a, W, O> {
    fmt: &'a mut fmt::Formatter<W, O>,
    result: fmt::Result,
    has_fields: bool,
}

impl<'a, W: Write, O: FmtOpts> DebugInner<'a, W, O> {
    fn entry(&mut self, entry: &impl Debug) {
        self.result = self.result.and_then(|_| {
            if self.is_pretty() {
                if !self.has_fields {
                    self.fmt.write_str("\n")?;
                }
                let mut slot = None;
                let mut state = Default::default();
                let mut writer = PadAdapter::wrap(self.fmt, &mut slot, &mut state);
                entry.fmt(&mut writer)?;
                writer.write_str(",\n")
            } else {
                if self.has_fields {
                    self.fmt.write_str(", ")?
                }
                entry.fmt(self.fmt)
            }
        });

        self.has_fields = true;
    }

    fn is_pretty(&self) -> bool {
        self.fmt.alternate()
    }
}

#[must_use = "must eventually call `finish()` on Debug builders"]
#[allow(missing_debug_implementations)]
pub struct DebugSet<'a, W, O> {
    inner: DebugInner<'a, W, O>,
}

pub(super) fn debug_set_new<'a, W: Write, O: FmtOpts>(
    fmt: &'a mut fmt::Formatter<W, O>,
) -> DebugSet<'a, W, O> {
    let result = fmt.write_str("{");
    DebugSet {
        inner: DebugInner {
            fmt,
            result,
            has_fields: false,
        },
    }
}

impl<'a, W: Write, O: FmtOpts> DebugSet<'a, W, O> {
    pub fn entry(&mut self, entry: &impl Debug) -> &mut Self {
        self.inner.entry(entry);
        self
    }

    pub fn entries<D, I>(&mut self, entries: I) -> &mut Self
    where
        D: fmt::Debug,
        I: IntoIterator<Item = D>,
    {
        for entry in entries {
            self.entry(&entry);
        }
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.inner
            .result
            .and_then(|_| self.inner.fmt.write_str("}"))
    }
}

#[must_use = "must eventually call `finish()` on Debug builders"]
#[allow(missing_debug_implementations)]
pub struct DebugList<'a, W, O> {
    inner: DebugInner<'a, W, O>,
}

pub(super) fn debug_list_new<'a, W: Write, O: FmtOpts>(
    fmt: &'a mut fmt::Formatter<W, O>,
) -> DebugList<'a, W, O> {
    let result = fmt.write_str("[");
    DebugList {
        inner: DebugInner {
            fmt,
            result,
            has_fields: false,
        },
    }
}

impl<'a, W: Write, O: FmtOpts> DebugList<'a, W, O> {
    pub fn entry(&mut self, entry: &impl Debug) -> &mut Self {
        self.inner.entry(entry);
        self
    }

    pub fn entries<D, I>(&mut self, entries: I) -> &mut Self
    where
        D: fmt::Debug,
        I: IntoIterator<Item = D>,
    {
        for entry in entries {
            self.entry(&entry);
        }
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.inner
            .result
            .and_then(|_| self.inner.fmt.write_str("]"))
    }
}

#[must_use = "must eventually call `finish()` on Debug builders"]
#[allow(missing_debug_implementations)]
pub struct DebugMap<'a, W, O> {
    fmt: &'a mut fmt::Formatter<W, O>,
    result: fmt::Result,
    has_fields: bool,
    has_key: bool,
    // The state of newlines is tracked between keys and values
    state: PadAdapterState,
}

pub(super) fn debug_map_new<'a, 'b, W: Write, O: FmtOpts>(
    fmt: &'a mut fmt::Formatter<W, O>,
) -> DebugMap<'a, W, O> {
    let result = fmt.write_str("{");
    DebugMap {
        fmt,
        result,
        has_fields: false,
        has_key: false,
        state: Default::default(),
    }
}

impl<'a, W: Write, O: FmtOpts> DebugMap<'a, W, O> {
    pub fn entry(&mut self, key: &impl Debug, value: &impl Debug) -> &mut Self {
        self.key(key).value(value)
    }

    pub fn key(&mut self, key: &impl Debug) -> &mut Self {
        self.result = self.result.and_then(|_| {
            assert!(
                !self.has_key,
                "attempted to begin a new map entry \
                                    without completing the previous one"
            );

            if self.is_pretty() {
                if !self.has_fields {
                    self.fmt.write_str("\n")?;
                }
                let mut slot = None;
                self.state = Default::default();
                let mut writer = PadAdapter::wrap(self.fmt, &mut slot, &mut self.state);
                key.fmt(&mut writer)?;
                writer.write_str(": ")?;
            } else {
                if self.has_fields {
                    self.fmt.write_str(", ")?
                }
                key.fmt(self.fmt)?;
                self.fmt.write_str(": ")?;
            }

            self.has_key = true;
            Ok(())
        });

        self
    }

    pub fn value(&mut self, value: impl Debug) -> &mut Self {
        self.result = self.result.and_then(|_| {
            assert!(
                self.has_key,
                "attempted to format a map value before its key"
            );

            if self.is_pretty() {
                let mut slot = None;
                let mut writer = PadAdapter::wrap(self.fmt, &mut slot, &mut self.state);
                value.fmt(&mut writer)?;
                writer.write_str(",\n")?;
            } else {
                value.fmt(self.fmt)?;
            }

            self.has_key = false;
            Ok(())
        });

        self.has_fields = true;
        self
    }

    pub fn entries<K, V, I>(&mut self, entries: I) -> &mut Self
    where
        K: fmt::Debug,
        V: fmt::Debug,
        I: IntoIterator<Item = (K, V)>,
    {
        for (k, v) in entries {
            self.entry(&k, &v);
        }
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.result.and_then(|_| {
            assert!(
                !self.has_key,
                "attempted to finish a map with a partial entry"
            );

            self.fmt.write_str("}")
        })
    }

    fn is_pretty(&self) -> bool {
        self.fmt.alternate()
    }
}
