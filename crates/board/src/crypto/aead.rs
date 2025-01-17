// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Authenticated Encryption with Associated Data.

use generic_array::{ArrayLength, GenericArray};
#[cfg(feature = "internal-aead")]
pub use software::*;

use crate::{Error, Support, Unsupported};

#[derive(Copy, Clone)]
pub struct AeadSupport {
    pub no_copy: bool,
    pub in_place_no_copy: bool,
}

impl From<AeadSupport> for bool {
    fn from(value: AeadSupport) -> Self {
        value.no_copy || value.in_place_no_copy
    }
}

/// Elliptic-curve cryptography interface.
pub trait Api<Key, Iv, Tag>: Support<AeadSupport>
where
    Key: ArrayLength<u8>,
    Iv: ArrayLength<u8>,
    Tag: ArrayLength<u8>,
{
    /// Encrypts and authenticates a clear text with associated data given a key and IV.
    ///
    /// The clear- and cipher-texts must have the same length. If the clear text is omitted, then
    /// the cipher text is encrypted in place.
    fn encrypt(
        key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], clear: Option<&[u8]>, cipher: &mut [u8],
        tag: &mut Array<Tag>,
    ) -> Result<(), Error>;

    /// Decrypts and authenticates a cipher text with associated data given a key and IV.
    ///
    /// The cipher- and clear-texts must have the same length. If the cipher text is omitted, then
    /// the clear text is decrypted in place.
    fn decrypt(
        key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], cipher: Option<&[u8]>, tag: &Array<Tag>,
        clear: &mut [u8],
    ) -> Result<(), Error>;
}

pub type Array<N> = GenericArray<u8, N>;

impl Support<AeadSupport> for Unsupported {
    const SUPPORT: AeadSupport = AeadSupport { no_copy: false, in_place_no_copy: false };
}

impl<Key, Iv, Tag> Api<Key, Iv, Tag> for Unsupported
where
    Key: ArrayLength<u8>,
    Iv: ArrayLength<u8>,
    Tag: ArrayLength<u8>,
{
    fn encrypt(
        _: &Array<Key>, _: &Array<Iv>, _: &[u8], _: Option<&[u8]>, _: &mut [u8], _: &mut Array<Tag>,
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn decrypt(
        _: &Array<Key>, _: &Array<Iv>, _: &[u8], _: Option<&[u8]>, _: &Array<Tag>, _: &mut [u8],
    ) -> Result<(), Error> {
        unreachable!()
    }
}

#[cfg(feature = "internal-aead")]
mod software {
    use aead::{AeadCore, AeadInPlace};
    use crypto_common::{KeyInit, KeySizeUser};

    use super::*;

    impl<T: AeadInPlace> Support<AeadSupport> for T {
        const SUPPORT: AeadSupport = AeadSupport { no_copy: false, in_place_no_copy: true };
    }

    impl<Key, Iv, Tag, T> Api<Key, Iv, Tag> for T
    where
        T: KeyInit + AeadInPlace,
        T: KeySizeUser<KeySize = Key>,
        T: AeadCore<NonceSize = Iv, TagSize = Tag>,
        Key: ArrayLength<u8>,
        Iv: ArrayLength<u8>,
        Tag: ArrayLength<u8>,
    {
        fn encrypt(
            key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], clear: Option<&[u8]>, cipher: &mut [u8],
            tag: &mut Array<Tag>,
        ) -> Result<(), Error> {
            let aead = T::new(key);
            if let Some(clear) = clear {
                cipher.copy_from_slice(clear);
            }
            tag.copy_from_slice(
                &aead.encrypt_in_place_detached(iv, aad, cipher).map_err(|_| Error::World)?,
            );
            Ok(())
        }

        fn decrypt(
            key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], cipher: Option<&[u8]>, tag: &Array<Tag>,
            clear: &mut [u8],
        ) -> Result<(), Error> {
            let aead = T::new(key);
            if let Some(cipher) = cipher {
                clear.copy_from_slice(cipher);
            }
            aead.decrypt_in_place_detached(iv, aad, clear, tag).map_err(|_| Error::World)
        }
    }
}
