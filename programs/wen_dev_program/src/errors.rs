use anchor_lang::prelude::*;
pub use WenDevError::*;

#[error_code]
pub enum WenDevError {
    // 6000
    #[msg("ValueTooSmall")]
    ValueTooSmall,

    // 6001
    #[msg("ValueTooLarge")]
    ValueTooLarge,

    // 6002
    #[msg("ValueInvalid")]
    ValueInvalid,

    // 6003
    #[msg("SerializationFailed")]
    SerializationFailed,

    // 6004
    #[msg("IncorrectAuthority")]
    IncorrectAuthority,

    // 6005
    #[msg("NothingToDo")]
    NothingToDo,

    // 6006
    #[msg("IncorrectLaunchPhase")]
    IncorrectLaunchPhase,

    // 6007
    #[msg("MissingValueToCreateSniper")]
    MissingValueToCreateSniper,

    // 6008
    #[msg("BuyLamportsExceedsMaxSnipeAmount")]
    BuyLamportsExceedsMaxSnipeAmount,

    // 6009
    #[msg("The launch is not completed yet.")]
    NotCompleted,
}
