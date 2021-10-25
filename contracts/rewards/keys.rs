pub mod pool {
    pub const CLAIMED:      &[u8] = b"/pool/claimed";
    pub const CLOSED:       &[u8] = b"/pool/closed";
    pub const COOLDOWN:     &[u8] = b"/pool/cooldown";
    pub const CREATED:      &[u8] = b"/pool/created";
    pub const LIFETIME:     &[u8] = b"/pool/lifetime";
    pub const LIQUID:       &[u8] = b"/pool/not_empty";
    pub const LOCKED:       &[u8] = b"/pool/balance";
    pub const LP_TOKEN:     &[u8] = b"/pool/lp_token";
    pub const RATIO:        &[u8] = b"/pool/ratio";
    pub const REWARD_TOKEN: &[u8] = b"/pool/reward_token";
    pub const REWARD_VK:    &[u8] = b"/pool/reward_vk";
    pub const SEEDED:       &[u8] = b"/pool/created";
    pub const SELF:         &[u8] = b"/pool/self";
    pub const THRESHOLD:    &[u8] = b"/pool/threshold";
    pub const TIMESTAMP:    &[u8] = b"/pool/updated";
}

pub mod user {
    pub const CLAIMED:   &[u8] = b"/user/claimed/";
    pub const COOLDOWN:  &[u8] = b"/user/cooldown/";
    pub const EXISTED:   &[u8] = b"/user/existed/";
    pub const LIFETIME:  &[u8] = b"/user/lifetime/";
    pub const LOCKED:    &[u8] = b"/user/current/";
    pub const PRESENT:   &[u8] = b"/user/present/";
    pub const TIMESTAMP: &[u8] = b"/user/updated/";
}
