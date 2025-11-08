use crate::domain::common::CoreError;


pub struct IsHealthy(bool);

impl IsHealthy {
    
    pub fn new() -> Self {
        Self(false)
    }

    pub fn value(&self) -> bool {
        self.0
    }
    
    pub fn to_result(&self) -> Result<Self, CoreError> {
            if self.value() {
                Ok(IsHealthy(self.0))
            } else {
                Err(CoreError::Unhealthy)
            }
        }
}