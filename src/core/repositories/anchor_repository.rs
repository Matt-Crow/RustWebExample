use crate::core::models::anchor::Anchor;

/// designates something as an interface into a backing store containing anchors
pub trait AnchorRepository {

    /// creates or updates the given anchor in the backing store, then returns
    /// either the updated anchor or an error message 
    fn store(&mut self, anchor: &Anchor) -> Result<Anchor, String>;

    /// retrieves the anchor with the given ID from the backing store, then
    /// returns either the anchor, nothing, or an error message
    fn get_by_id(&self, id: u32) -> Result<Option<Anchor>, String>;
}