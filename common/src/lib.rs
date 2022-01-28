pub mod api;
pub mod router;
pub mod id {
    pub type MachineId = uuid::Uuid;
    pub type DeviceId = uuid::Uuid;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
