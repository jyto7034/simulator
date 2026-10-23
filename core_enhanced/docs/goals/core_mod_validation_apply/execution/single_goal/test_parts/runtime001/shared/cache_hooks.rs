//! Independently authored cfg(test)-only boundary controls. These never produce
//! Lab outcomes, accepted records or approval capabilities. Every invocation is
//! confined to the current test thread and consumed once.
use super::{JobId,LabOutcome};
use std::cell::RefCell;

type AfterLab=Box<dyn FnOnce(JobId,Option<(&LabOutcome,bool)>)>;
thread_local! {
    static AFTER_LAB:RefCell<Option<AfterLab>>=RefCell::new(None);
    static POST_RENAME:RefCell<Option<String>>=RefCell::new(None);
}
pub(crate) struct Reset;
impl Drop for Reset {
    fn drop(&mut self){
        AFTER_LAB.with(|slot|{slot.borrow_mut().take();});
        POST_RENAME.with(|slot|{slot.borrow_mut().take();});
    }
}
pub(crate) fn install_after_lab(callback:impl FnOnce(JobId,Option<(&LabOutcome,bool)>)+ 'static)->Reset {
    AFTER_LAB.with(|slot|{assert!(slot.borrow().is_none(),"boundary hook already installed");
        *slot.borrow_mut()=Some(Box::new(callback));});Reset
}
pub(crate) fn install_post_rename_fault(reason:&str)->Reset {
    POST_RENAME.with(|slot|{assert!(slot.borrow().is_none(),"durability fault already installed");
        *slot.borrow_mut()=Some(reason.to_owned());});Reset
}
pub(crate) fn after_lab(id:JobId,result:Option<(&LabOutcome,bool)>){
    let callback=AFTER_LAB.with(|slot|slot.borrow_mut().take());
    if let Some(callback)=callback{callback(id,result);}
}
pub(crate) fn post_rename_fault()->Option<String>{POST_RENAME.with(|slot|slot.borrow_mut().take())}
