use crate::models::anchor::Anchor;

/// designates something as an interface into a backing store containing anchors
pub trait AnchorRepository {

    /// creates or updates the given anchor in the backing store, then returns
    /// either the updated anchor or an error message 
    fn store(&mut self, anchor: &Anchor) -> Result<Anchor, String>;

    /// retrieves all anchors from the backing store, then returns either all
    /// the anchors or an error message
    fn get_all(&self) -> Result<Vec<Anchor>, String>;

    /// retrieves the anchor with the given ID from the backing store, then
    /// returns either the anchor, nothing, or an error message
    fn get_by_id(&self, id: u128) -> Result<Option<Anchor>, String>;

    /// removes the anchor with the given ID from the backing store, then 
    /// returns a boolean indicating if the anchor existed or an error message
    fn remove_by_id(&mut self, id: u128) -> Result<bool, String>;
}