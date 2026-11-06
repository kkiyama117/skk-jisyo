// CONTRACT: do not edit
//! Public data types of the `jisyo` crate. `SPEC.md` gives the rules that use them.

use std::fmt;

/// The text encoding of a dictionary file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// JIS X 0201 kana, JIS X 0208, and JIS X 0212, mapped as `data/euc-jp.txt`.
    EucJp,
    /// JIS X 0201 kana and JIS X 0213:2004 planes 1 and 2, mapped as `data/euc-jis-2004-std.txt`.
    EucJis2004,
    Utf8,
    Utf16Le,
}

/// The line terminator of one line. Only the last line of a file can have `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Eol {
    Lf,
    CrLf,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Section {
    OkuriAri,
    OkuriNasi,
}

/// One candidate: `word` or `word;annotation`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Candidate {
    pub word: String,
    pub annotation: Option<String>,
}

/// `[okuri/cand/.../]` inside an okuri-ari entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OkuriBlock {
    pub okuri: String,
    pub candidates: Vec<Candidate>,
}

/// One entry line: `midashi /cand/.../[okuri/cand/.../]/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub midashi: String,
    pub candidates: Vec<Candidate>,
    pub blocks: Vec<OkuriBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineKind {
    Blank,
    Comment(String),
    Marker(Section),
    Entry(Entry),
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub kind: LineKind,
    pub eol: Eol,
}

/// A dictionary file, kept so that writing it back gives the same bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Jisyo {
    pub encoding: Encoding,
    pub bom: bool,
    pub lines: Vec<Line>,
}

/// What a dictionary means: the entries of each section, each midashi once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dict {
    pub okuri_ari: Vec<Entry>,
    pub okuri_nasi: Vec<Entry>,
}

/// The IME whose user-dictionary format an export writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Corvus,
    Fcitx5,
    Skkeleton,
}

/// How an export writes a `Dict` as a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub encoding: Encoding,
    pub bom: bool,
    pub eol: Eol,
    /// The whole first line, without its terminator, or `None` for no such line.
    pub cookie: Option<&'static str>,
    pub blocks: bool,
}

/// What a learning event does to one candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LearnOp {
    /// A new word was registered. It goes to the front of the list.
    Add,
    /// A word was chosen. It moves to the front of the list.
    Promote,
    /// A word was purged from the list.
    Delete,
    /// Only the annotation changed. The position stays.
    Annotate,
}

/// A learning event: the engine emits one each time it learns, and an import from an IME
/// file produces the same events. It addresses one candidate of one list. `okuri` is `None`
/// for the main list of the entry and `Some(okuri)` for a `[okuri/.../]` block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnEvent {
    pub op: LearnOp,
    pub section: Section,
    pub midashi: String,
    pub okuri: Option<String>,
    pub candidate: Candidate,
}

/// A learning event as one line of a machine's log. `time` is milliseconds since the Unix epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub machine: String,
    pub seq: u64,
    pub time: u64,
    pub event: LearnEvent,
}

/// The only source of the current time, used only outside the core. Milliseconds since the Unix epoch.
pub trait Clock {
    fn now_ms(&self) -> u64;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    Io,
    Decode,
    Encode,
    UnknownCoding,
    NotRoundTrip,
    BadField,
    BadMachineId,
    LogHeader,
    LogSyntax,
    LogSeq,
    LogTime,
    LogConflict,
    PartialTail,
    TooManyDeletes,
    TargetMissing,
    Unparsed,
    Usage,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::Io => "io",
            ErrorCode::Decode => "decode",
            ErrorCode::Encode => "encode",
            ErrorCode::UnknownCoding => "unknown-coding",
            ErrorCode::NotRoundTrip => "not-round-trip",
            ErrorCode::BadField => "bad-field",
            ErrorCode::BadMachineId => "bad-machine-id",
            ErrorCode::LogHeader => "log-header",
            ErrorCode::LogSyntax => "log-syntax",
            ErrorCode::LogSeq => "log-seq",
            ErrorCode::LogTime => "log-time",
            ErrorCode::LogConflict => "log-conflict",
            ErrorCode::PartialTail => "partial-tail",
            ErrorCode::TooManyDeletes => "too-many-deletes",
            ErrorCode::TargetMissing => "target-missing",
            ErrorCode::Unparsed => "unparsed",
            ErrorCode::Usage => "usage",
        }
    }
}

/// Every error of the crate. It never holds dictionary text, only positions and counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: ErrorCode,
    /// 1-based line number.
    pub line: Option<u64>,
    /// 0-based byte offset.
    pub offset: Option<u64>,
    pub count: Option<u64>,
    pub io: Option<std::io::ErrorKind>,
}

impl Error {
    pub fn new(code: ErrorCode) -> Error {
        Error {
            code,
            line: None,
            offset: None,
            count: None,
            io: None,
        }
    }

    pub fn line(mut self, line: u64) -> Error {
        self.line = Some(line);
        self
    }

    pub fn offset(mut self, offset: u64) -> Error {
        self.offset = Some(offset);
        self
    }

    pub fn count(mut self, count: u64) -> Error {
        self.count = Some(count);
        self
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        let mut err = Error::new(ErrorCode::Io);
        err.io = Some(e.kind());
        err
    }
}

/// `<code>`, then ` line <n>`, ` offset <n>`, ` count <n>`, ` io <kind>` for each field that is set, in that order.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code.as_str())?;
        if let Some(n) = self.line {
            write!(f, " line {n}")?;
        }
        if let Some(n) = self.offset {
            write!(f, " offset {n}")?;
        }
        if let Some(n) = self.count {
            write!(f, " count {n}")?;
        }
        if let Some(k) = self.io {
            write!(f, " io {k:?}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}
