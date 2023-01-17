use crate::content::content_enum;

content_enum!
{
	pub enum Type / Fluid for u16 | TryFromU16Error
	{
		Water => "water",
		Slag => "slag",
		Oil => "oil",
		Cryofluid => "cryofluid",
		Neoplasm => "neoplasm",
		Arkycite => "arkycite",
		Gallium => "gallium",
		Ozone => "ozone",
		Hydrogen => "hydrogen",
		Nitrogen => "nitrogen",
		Cyanogen => "cyanogen",
	}
}
