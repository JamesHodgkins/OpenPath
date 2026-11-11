//! Icons. Each is its own constant, named after the command or part of the
//! window it is for (`DRAW_LINE`, `LAYER_PANEL_FROZEN`), so each can be redrawn
//! on its own: `assets/icons/<name>.svg`, with `_` as `-`. See [`crate::icon_set`].
//!
//! The value of a constant identifies the icon and is not a character of the
//! icon font; [`glyph`] gives the font's glyph (Lucide, lucide-static 0.469.0,
//! ISC licence, see assets/fonts/LICENSE-Lucide.txt) that is drawn when an icon
//! has no SVG file. Two icons may share a glyph.
//!
//! Add an icon by adding a line below with the Lucide glyph to start from, then
//! run `python scripts/export_icons.py`.

/// Where icon identifiers start, in a Unicode private use plane.
const BASE: u32 = 0xF0000;

const fn id(index: u32) -> char {
    match char::from_u32(BASE + index) {
        Some(c) => c,
        None => panic!("too many icons"),
    }
}

/// The font glyph drawn for `icon` when it has no drawing.
pub fn glyph(icon: char) -> char {
    (icon as u32)
        .checked_sub(BASE)
        .and_then(|i| GLYPHS.get(i as usize))
        .copied()
        .unwrap_or(icon)
}

/// Declares the icons, [`ALL`] and [`GLYPHS`].
macro_rules! icons {
    ($($(#[$doc:meta])* $name:ident = $glyph:literal;)*) => {
        #[allow(non_camel_case_types, clippy::enum_variant_names)]
        #[derive(Clone, Copy)]
        enum Index { $($name),* }

        $($(#[$doc])* pub const $name: char = id(Index::$name as u32);)*

        /// Every icon with its constant's name, which names its SVG file
        /// (`DRAW_LINE` is `draw-line.svg`).
        pub const ALL: &[(&str, char)] = &[$((stringify!($name), $name)),*];

        /// The font glyph of each icon, in the order of [`ALL`].
        const GLYPHS: &[char] = &[$($glyph),*];
    };
}

icons! {
    ANNOTATE_LEADER = '\u{e051}';
    ANNOTATE_MLEADER = '\u{e497}';
    ANNOTATE_MLEADERSTYLE = '\u{e579}';
    ANNOTATE_MLEADER_ADD = '\u{e140}';
    ANNOTATE_MLEADER_ALIGN = '\u{e041}';
    ANNOTATE_MLEADER_COLLECT = '\u{e443}';
    ANNOTATE_MLEADER_REMOVE = '\u{e11f}';
    ANNOTATE_MTEXT = '\u{e197}';
    ANNOTATE_TABLE = '\u{e180}';
    ANNOTATE_TABLESTYLE = '\u{e180}';
    ANNOTATE_TEXT = '\u{e264}';
    ANNOTATE_TEXTSTYLE = '\u{e3dc}';
    ASSISTANT_ADD = '\u{e140}';
    ASSISTANT_BURGER = '\u{e118}';
    ASSISTANT_COPY ='\u{e0a2}';
    ASSISTANT_HISTORY = '\u{e1f4}';
    ASSISTANT_MENU = '\u{e071}';
    ASSISTANT_NEW_CHAT = '\u{e410}';
    ASSISTANT_SEND = '\u{e04e}';
    ASSISTANT_STOP = '\u{e16a}';
    BLOCK_ATTDEF = '\u{e182}';
    BLOCK_ATTEDIT = '\u{e133}';
    BLOCK_ATTSYNC = '\u{e148}';
    BLOCK_CLOSE = '\u{e1b1}';
    BLOCK_CREATE = '\u{e2ac}';
    BLOCK_EATTEDIT = '\u{e10b}';
    BLOCK_EDIT = '\u{e1f8}';
    BLOCK_INSERT = '\u{e065}';
    BLOCK_PURGE = '\u{e28e}';
    BLOCK_SAVE = '\u{e150}';
    COMBO_ARROW = '\u{e071}';
    COMMAND_LINE_COLLAPSE = '\u{e071}';
    COMMAND_LINE_EXPAND = '\u{e073}';
    CONTEXT_DESELECT_ALL = '\u{e178}';
    CONTEXT_EDIT_SELECT_ALL = '\u{e1c2}';
    CONTEXT_FIND = '\u{e154}';
    CONTEXT_MODIFY_ERASE = '\u{e18d}';
    CONTEXT_QUICKCALC = '\u{e1bb}';
    CONTEXT_REPEAT = '\u{e149}';
    CONTEXT_SELECT_SIMILAR = '\u{e3ff}';
    CONTEXT_SUBMENU = '\u{e073}';
    CONTEXT_VIEW_PANEL_PROPERTIES = '\u{e299}';
    CONTEXT_VIEW_ZOOM_EXTENTS = '\u{e1b5}';
    DIALOG_CLOSE = '\u{e1b1}';
    DIM_ALIGNED = '\u{e1c3}';
    DIM_ANGULAR = '\u{e4f1}';
    DIM_ARC = '\u{e4b4}';
    DIM_BASELINE = '\u{e270}';
    DIM_CENTRE = '\u{e348}';
    DIM_CONTINUE = '\u{e596}';
    DIM_DISASSOCIATE = '\u{e109}';
    DIM_EDIT = '\u{e4f5}';
    DIM_JOGGED = '\u{e542}';
    DIM_JOGLINE = '\u{e611}';
    DIM_ORDINATE = '\u{e0b0}';
    DIM_REASSOCIATE = '\u{e108}';
    DIM_SPACE = '\u{e27a}';
    DIM_TEXTEDIT = '\u{e264}';
    DIM_DIAMETER = '\u{e52a}';
    DIM_DIM = '\u{e14e}';
    DIM_LINEAR = '\u{e1c5}';
    DIM_RADIUS = '\u{e531}';
    DIM_STYLE = '\u{e14e}';
    DOCUMENT_TAB_CLOSE = '\u{e1b1}';
    DOCUMENT_TAB_HOME = '\u{e0f8}';
    DOCUMENT_TAB_NEW = '\u{e140}';
    DRAW_ARC_3P = '\u{e4c6}';
    DRAW_ARC_SCE = '\u{e4c6}';
    DRAW_BOUNDARY = '\u{e1ca}';
    DRAW_CIRCLE_2P = '\u{e07a}';
    DRAW_CIRCLE_3P = '\u{e07a}';
    DRAW_CIRCLE_3T = '\u{e07a}';
    DRAW_CIRCLE_DIAMETER = '\u{e07a}';
    DRAW_CIRCLE_RADIUS = '\u{e07a}';
    DRAW_CIRCLE_TTR = '\u{e07a}';
    /// Lucide's ellipsis: marks along a line.
    DRAW_DIVIDE = '\u{e0ba}';
    /// Lucide's donut.
    DRAW_DONUT = '\u{e4c0}';
    /// Lucide has no ellipse; its egg is the nearest single oval outline.
    DRAW_ELLIPSE = '\u{e25c}';
    DRAW_GRADIENT = '\u{e5a0}';
    /// A square with a diagonal: the nearest Lucide has to hatching.
    DRAW_HATCH = '\u{e177}';
    DRAW_LINE = '\u{e11f}';
    /// Lucide's marks between two ends.
    DRAW_MEASURE = '\u{e596}';
    /// Lucide's dot.
    DRAW_POINT = '\u{e453}';
    DRAW_POLYGON = '\u{e0f6}';
    DRAW_POLYLINE = '\u{e38e}';
    /// Lucide's arrow up and right: a line that leaves a point.
    DRAW_RAY = '\u{e497}';
    DRAW_RECTANGLE = '\u{e16a}';
    DRAW_SPLINE = '\u{e38e}';
    /// Lucide's slash: a line that runs on both ways.
    DRAW_XLINE = '\u{e521}';
    DROPDOWN_ARROW = '\u{e071}';
    DYNAMIC_INPUT_LOCKED = '\u{e10e}';
    DYNAMIC_INPUT_OPTIONS = '\u{e071}';
    EDIT_COPY_BASE = '\u{e401}';
    EDIT_COPY_CLIP = '\u{e0a2}';
    EDIT_CUT = '\u{e151}';
    EDIT_PASTE = '\u{e3eb}';
    EDIT_PASTE_BLOCK = '\u{e065}';
    EDIT_PASTE_ORIGINAL = '\u{e1da}';
    EDIT_REDO = '\u{e29f}';
    EDIT_SELECT_ALL = '\u{e50d}';
    EDIT_UNDO = '\u{e2a0}';
    FILE_NEW = '\u{e0c4}';
    FILE_OPEN = '\u{e246}';
    FILE_SAVE = '\u{e150}';
    FORMAT_LINETYPE = '\u{e0ba}';
    FORMAT_PROPERTIES = '\u{e0d0}';
    FORMAT_UNITS = '\u{e14e}';
    GALLERY_LANDSCAPE = '\u{e379}';
    GALLERY_PORTRAIT = '\u{e37a}';
    GALLERY_PRESET = '\u{e0f9}';
    GROUP_BOUNDING_BOX = '\u{e1ca}';
    GROUP_EDIT = '\u{e175}';
    GROUP_GROUP = '\u{e468}';
    GROUP_MANAGER = '\u{e10b}';
    GROUP_SELECTION = '\u{e201}';
    GROUP_UNGROUP = '\u{e46b}';
    HOME_MORE = '\u{e0ba}';
    HOME_PIN = '\u{e258}';
    HOME_TEMPLATE = '\u{e206}';
    HOME_VIEW_GRID = '\u{e104}';
    HOME_VIEW_LIST = '\u{e10b}';
    INQUIRY_ANGLE = '\u{e4f1}';
    INQUIRY_AREA = '\u{e16a}';
    INQUIRY_DISTANCE = '\u{e14e}';
    INQUIRY_ID = '\u{e1da}';
    INQUIRY_RADIUS = '\u{e531}';
    INQUIRY_VOLUME = '\u{e065}';
    INSERT_EXPORT = '\u{e19d}';
    INSERT_FIELD = '\u{e264}';
    INSERT_IMPORT = '\u{e0b6}';
    INSERT_UPDATEFIELD = '\u{e4b6}';
    LAYER_FREEZE = '\u{e168}';
    LAYER_HIDE = '\u{e207}';
    LAYER_ISOLATE = '\u{e29d}';
    LAYER_LIST_FROZEN = '\u{e168}';
    LAYER_LIST_LOCKED = '\u{e10e}';
    LAYER_LIST_OFF = '\u{e207}';
    LAYER_LIST_ON = '\u{e1c1}';
    LAYER_LIST_THAWED = '\u{e17b}';
    LAYER_LIST_UNLOCKED = '\u{e10f}';
    LAYER_LOCK = '\u{e10e}';
    LAYER_MAKE_CURRENT = '\u{e52e}';
    LAYER_MATCH = '\u{e13e}';
    LAYER_PANEL_CURRENT = '\u{e070}';
    LAYER_PANEL_FROZEN = '\u{e168}';
    LAYER_PANEL_LOCKED = '\u{e10e}';
    LAYER_PANEL_OFF = '\u{e0bf}';
    LAYER_PANEL_ON = '\u{e0be}';
    LAYER_PANEL_UNLOCKED = '\u{e10f}';
    LAYER_PANEL_VP_FROZEN = '\u{e168}';
    LAYER_PROPERTIES = '\u{e52d}';
    LAYER_SHOW = '\u{e1c1}';
    LAYER_STATES = '\u{e10b}';
    LAYER_THAW_ALL = '\u{e17b}';
    LAYER_UNISOLATE = '\u{e256}';
    LAYER_UNLOCK = '\u{e10f}';
    MANAGE_SETTINGS = '\u{e157}';
    MODIFY_ARRAY_PATH = '\u{e282}';
    MODIFY_ARRAY_POLAR = '\u{e4b6}';
    MODIFY_ARRAY_RECT = '\u{e503}';
    MODIFY_BLEND = '\u{e38e}';
    MODIFY_BREAK = '\u{e4ed}';
    MODIFY_CHAMFER = '\u{e4f1}';
    MODIFY_COPY = '\u{e0a2}';
    MODIFY_ERASE = '\u{e28e}';
    MODIFY_EXPLODE = '\u{e46b}';
    MODIFY_EXTEND = '\u{e45d}';
    MODIFY_FILLET = '\u{e0a6}';
    MODIFY_HATCHEDIT = '\u{e177}';
    MODIFY_JOIN = '\u{e443}';
    MODIFY_LENGTHEN = '\u{e1c5}';
    MODIFY_MATCHPROP = '\u{e2e6}';
    MODIFY_MIRROR = '\u{e360}';
    MODIFY_MOVE = '\u{e124}';
    MODIFY_OFFSET = '\u{e27b}';
    MODIFY_PEDIT = '\u{e546}';
    MODIFY_ROTATE = '\u{e14c}';
    MODIFY_SCALE = '\u{e115}';
    MODIFY_STRETCH = '\u{e441}';
    MODIFY_TRIM = '\u{e151}';
    OSNAP_APPARENT_INTERSECTION = '\u{e0ec}';
    OSNAP_CENTRE = '\u{e0ec}';
    OSNAP_END = '\u{e0ec}';
    OSNAP_EXTENSION = '\u{e0ec}';
    OSNAP_GEO_CENTRE = '\u{e0ec}';
    OSNAP_INSERTION = '\u{e0ec}';
    OSNAP_INTERSECTION = '\u{e0ec}';
    OSNAP_MID = '\u{e0ec}';
    OSNAP_NEAREST = '\u{e0ec}';
    OSNAP_NODE = '\u{e0ec}';
    OSNAP_PARALLEL = '\u{e0ec}';
    OSNAP_PERPENDICULAR = '\u{e0ec}';
    OSNAP_QUADRANT = '\u{e0ec}';
    OSNAP_TANGENT = '\u{e0ec}';
    PANEL_SECTION_CLOSED = '\u{e073}';
    PANEL_SECTION_OPEN = '\u{e071}';
    POWER_SELECT_PICK = '\u{e1c2}';
    PROPERTIES_COLOR = '\u{e1dc}';
    PROPERTIES_LINETYPE = '\u{e1bc}';
    PROPERTIES_LINEWEIGHT = '\u{e040}';
    RIBBON_PANEL_CLOSE = '\u{e074}';
    RIBBON_PANEL_OPEN = '\u{e071}';
    RIBBON_SPLIT_ARROW = '\u{e071}';
    SELECT_CROSSING_POLYGON = '\u{e1cd}';
    SELECT_FENCE = '\u{e542}';
    SELECT_POWER = '\u{e35a}';
    SELECT_WINDOW_POLYGON = '\u{e1ce}';
    TEXT_ALIGN_CENTER = '\u{e03c}';
    STATUS_GRID_DISPLAY = '\u{e0ec}';
    STATUS_GRID_SNAP = '\u{e2b4}';
    STATUS_ISO = '\u{e0ec}';
    STATUS_ORTHO = '\u{e0ec}';
    STATUS_OSNAP = '\u{e2b4}';
    STATUS_OTRACK = '\u{e0ec}';
    STATUS_POLAR = '\u{e0ec}';
    STATUS_SETTINGS = '\u{e0ec}';
    TEXT_ALIGN_JUSTIFY = '\u{e040}';
    TEXT_ALIGN_LEFT = '\u{e041}';
    TEXT_ALIGN_RIGHT = '\u{e042}';
    TEXT_LIST = '\u{e10b}';
    TEXT_REDO = '\u{e29f}';
    TEXT_UNDO = '\u{e2a0}';
    TOOLS_QUICKCALC = '\u{e1bb}';
    TOOL_SELECT = '\u{e1c2}';
    VIEW_ASSISTANT = '\u{e1ba}';
    VIEW_BASE_INVENTOR = '\u{e0c4}';
    VIEW_BASE_MODEL = '\u{e290}';
    VIEW_COMMAND_LINE = '\u{e379}';
    VIEW_GRID = '\u{e0ec}';
    VIEW_PAN = '\u{e1d6}';
    VIEW_PANEL_LAYERS = '\u{e52d}';
    VIEW_PANEL_PROPERTIES = '\u{e37a}';
    VIEW_RESET_LAYOUT = '\u{e2a0}';
    VIEW_SNAP = '\u{e2b4}';
    VIEW_ZOOM_EXTENTS = '\u{e115}';
    VIEW_ZOOM_IN = '\u{e1b5}';
    VIEW_ZOOM_OUT = '\u{e1b6}';
}
