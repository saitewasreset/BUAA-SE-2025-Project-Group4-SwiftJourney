macro_rules! impl_db_id_from_u64 {
    ($X:ty, $DbType:ty, $x_lower:expr) => {
        impl DbId for $X {
            type DbType = $DbType;

            fn to_db_value(&self) -> Self::DbType {
                u64::from(*self) as Self::DbType
            }

            fn from_db_value(value: Self::DbType) -> Result<Self, anyhow::Error> {
                <$X>::try_from(value)
                    .map_err(|_| anyhow::Error::msg(format!("Invalid {} id: {}", $x_lower, value)))
            }
        }
    };
}
