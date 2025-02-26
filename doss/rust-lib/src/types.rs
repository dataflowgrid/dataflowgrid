/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

#[derive(Debug)]
pub struct TypesError {
    message: String
}

pub struct Decimal {
    signed: bool,
    negative: bool,
    extension: i16, // equals 10^extension, can be positive or negative
    values: Box<[usize]>
}

impl Decimal {
    pub fn new(signed: bool, negative: bool, extension: i16, values: Box<[usize]>) -> Decimal {
        Decimal {
            signed,
            negative,
            extension,
            values
        }
    }

    /// creates a new Decimal with a usize value, not signed, not negative, no extension
    pub fn from_usize(value: usize) -> Decimal {
        Decimal {
            signed: false,
            negative: false,
            extension: 0,
            values: Box::new([value])
        }
    }

    /// creates a new Decimal with a isize value, no extension
    pub fn from_isize(value: isize) -> Decimal {
        if value < 0 {
            Decimal {
                signed: true,
                negative: true,
                extension: 0,
                values: Box::new([value.abs() as usize])
            }
        } else {
            Decimal {
                signed: true,
                negative: false,
                extension: 0,
                values: Box::new([value as usize])
            }
        }
    }

    pub fn get_usize(&self) -> Result<usize, TypesError> {
        if self.negative {
            return Err(TypesError {
                message: format!("negative value cannot be converted to usize")
            });
        }
        match self.values.len() {
            0 => Ok(0),
            1 => Ok(self.values[0]),
            _ => Err(TypesError {
                message: format!("Expected value smaller than usize but had {} fields", self.values.len())})
        }
    }

    pub fn get_isize(&self) -> Result<isize, TypesError> {
        match self.values.len() {
            0 => Ok(0),
            1 => {
                if self.negative {
                    Ok(-(self.values[0] as isize))
                } else {
                    Ok(self.values[0] as isize)
                }
            },
            _ => Err(TypesError {
                message: format!("Expected value smaller than isize but had {} fields", self.values.len())})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_usize() {
        let v = Decimal::from_usize(1);
        assert_eq!(v.get_usize().unwrap(), 1);
    }
}