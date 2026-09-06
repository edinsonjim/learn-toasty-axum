use toasty::Model;

#[derive(Debug, Model)]
pub struct Family {
    #[key]
    #[auto]
    pub id: u64,

    pub name: String,
    pub summary: Option<String>,

    pub deleted_at: Option<jiff::Timestamp>,
}
