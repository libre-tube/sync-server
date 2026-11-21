use std::sync::OnceLock;

use ::innertube::Innertube;
use async_trait::async_trait;

use crate::{
    database::DbPool,
    dto::{CreateVideo, ExtendedPublicPlaylist},
    models::Channel,
    validation::{
        basic::BasicValidator, db::DbValidator, innertube::InnertubeValidator, ytrss::RssValidator,
    },
};

mod basic;
mod db;
mod innertube;
mod ytrss;

static INNERTUBE: OnceLock<Innertube> = OnceLock::new();

#[derive(thiserror::Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Validation failed. Please try again later.")]
    UnableToValidate,
    #[error("provided metadata seems to be wrong")]
    Invalid,
    #[error("provided metadata seems to be wrong: {0}")]
    InvalidWithContext(String),
}

#[derive(Debug, Clone)]
pub enum ValidationResult<T> {
    Valid,
    ValidAfterChanges(T),
    /// Indicates that the validator decide to skip validating the data.
    Skip,
    Invalid(ValidationError),
}

impl<T> ValidationResult<T> {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn assign_changes(self, val: &mut T) -> Self {
        if let Self::ValidAfterChanges(changes) = self {
            *val = changes;
            Self::Valid
        } else {
            self
        }
    }

    pub fn into_result(self) -> Result<Option<T>, ValidationError> {
        match self {
            Self::Valid => Ok(None),
            Self::ValidAfterChanges(val) => Ok(Some(val)),
            Self::Skip => Err(ValidationError::UnableToValidate),
            Self::Invalid(err) => Err(err),
        }
    }
}

impl<T> From<ValidationError> for ValidationResult<T> {
    fn from(value: ValidationError) -> Self {
        Self::Invalid(value)
    }
}

#[async_trait]
pub trait Validator: Send + Sync + 'static {
    async fn is_channel_information_valid(&self, channel: &Channel) -> ValidationResult<Channel>;
    async fn is_video_information_valid(
        &self,
        video_data: &CreateVideo,
    ) -> ValidationResult<CreateVideo>;
    async fn is_playlist_information_valid(
        &self,
        playlist: &ExtendedPublicPlaylist,
    ) -> ValidationResult<ExtendedPublicPlaylist>;
}

#[derive(Default)]
pub struct CompositeValidator {
    validators: Vec<Box<dyn Validator>>,
}

impl CompositeValidator {
    fn validator(mut self, validator: impl Validator) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    fn maybe_validator(mut self, validator: Option<impl Validator>) -> Self {
        if let Some(validator) = validator {
            self.validators.push(Box::new(validator));
        }

        self
    }

    fn new() -> Self {
        Self::default()
    }
}

//             Skip           Invalid
// +----------+    +----------+   |  +----------+
// |Validator1|--->|Validator2| --+->|Validator3|
// +----------+    +----------+   |  +----------+
//                              break
macro_rules! validate {
    ($self:ident, $fn:ident, $val:expr) => {{
        let mut resp = ValidationResult::Skip;

        for validator in &$self.validators {
            resp = validator.$fn($val).await;
            if !matches!(resp, ValidationResult::Skip) {
                break;
            }
        }

        resp
    }};
}

#[async_trait]
impl Validator for CompositeValidator {
    async fn is_channel_information_valid(&self, channel: &Channel) -> ValidationResult<Channel> {
        validate!(self, is_channel_information_valid, channel)
    }

    async fn is_video_information_valid(
        &self,
        video_data: &CreateVideo,
    ) -> ValidationResult<CreateVideo> {
        validate!(self, is_video_information_valid, video_data)
    }

    async fn is_playlist_information_valid(
        &self,
        playlist: &ExtendedPublicPlaylist,
    ) -> ValidationResult<ExtendedPublicPlaylist> {
        validate!(self, is_playlist_information_valid, playlist)
    }
}

pub static VALIDATOR: OnceLock<CompositeValidator> = OnceLock::new();

pub fn init_validator(pool: DbPool) {
    let innertube = match Innertube::new() {
        Ok(it) => Some(INNERTUBE.get_or_init(|| it)),
        Err(err) => {
            log::error!("Innertube client initialization failed: {}", err);
            None
        }
    };

    let _ = VALIDATOR.set(
        CompositeValidator::new()
            .validator(BasicValidator)
            .validator(DbValidator::new(pool))
            .validator(RssValidator::new(innertube))
            .maybe_validator(innertube.map(InnertubeValidator::new)),
    );
}
