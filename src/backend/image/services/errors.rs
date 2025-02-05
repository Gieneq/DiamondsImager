// #[derive(Debug, thiserror::Error)]
// pub enum ProcessingFileError {
//     #[error("File width={actual} is too small, should be > {min}.")]
//     WidthTooSmall{
//         actual: u32,
//         min: u32,
//     },
    
//     #[error("File height={actual} is too small, should be > {min}.")]
//     HeightTooSmall{
//         actual: u32,
//         min: u32,
//     },
    
//     #[error("File width={actual} is too big, should be <= {max}.")]
//     WidthTooBig{
//         actual: u32,
//         max: u32,
//     },
    
//     #[error("File height={actual} is too big, should be <= {max}.")]
//     HeightTooBig{
//         actual: u32,
//         max: u32,
//     },
    
//     #[error("Some unknown error.")]
//     OtherError,
// }

#[derive(Debug, thiserror::Error)]
pub enum InputFileError {
    #[error("File is empty")]
    FileEmpty,

    #[error("File's name is missing")]
    FileNoname,

    #[error("File's size={actual} exceeded limit={limit}")]
    FileTooBig{
        actual: usize,
        limit: usize
    },

    #[error("File could not been saved")]
    FileNotSaved,
}