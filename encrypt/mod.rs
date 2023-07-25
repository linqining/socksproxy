
mod encrypt{
    pub fn aes256_cbc_encrypt(
        data: &[u8],
        key: &[u8; 32],
        iv: &[u8; 16],
    ) -> Result<Vec<u8>, SymmetricCipherError> {
        let mut encryptor = aes::cbc_encryptor(
            KeySize256,
            key, iv,
            PkcsPadding,
        );

        let mut buffer = [0; 4096];
        let mut write_buffer = RefWriteBuffer::new(&mut buffer);
        let mut read_buffer = RefReadBuffer::new(data);
        let mut final_result = Vec::new();

        loop {
            let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;
            final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
            match result {
                BufferUnderflow => break,
                _ => continue,
            }
        }

        Ok(final_result)
    }

    pub fn aes256_cbc_decrypt(
        data: &[u8],
        key: &[u8; 32],
        iv: &[u8; 16],
    ) -> Result<Vec<u8>, SymmetricCipherError> {
        let mut decryptor = aes::cbc_decryptor(
            KeySize256,
            key, iv,
            PkcsPadding,
        );

        let mut buffer = [0; 4096];
        let mut write_buffer = RefWriteBuffer::new(&mut buffer);
        let mut read_buffer = RefReadBuffer::new(data);
        let mut final_result = Vec::new();

        loop {
            let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
            final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
            match result {
                BufferUnderflow => break,
                _ => continue,
            }
        }

        Ok(final_result)
    }
}