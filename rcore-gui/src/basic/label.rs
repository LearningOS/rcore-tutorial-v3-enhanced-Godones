use alloc::string::String;
use alloc::sync::Arc;
use crate::{Component};
use crate::{Graphics, UPIntrFreeCell};

pub struct Label{
    inner: UPIntrFreeCell<LabelInner>,
}

struct LabelInner{
    text: String,
    graphic: Graphics,
    parent: Option<Arc<dyn Component>>,
}
