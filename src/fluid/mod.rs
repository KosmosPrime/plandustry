use crate::content::content_enum;

content_enum! {
    pub enum Type / Fluid for u16 | TryFromU16Error
    {
        "water",
        "slag",
        "oil",
        "cryofluid",
        "neoplasm",
        "arkycite",
        "gallium",
        "ozone",
        "hydrogen",
        "nitrogen",
        "cyanogen",
    }
}
