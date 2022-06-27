use postcard::de_flavors::Flavor;
use serde::de::DeserializeOwned;

pub struct BufferedIterator<'a, T> {
    iter: &'a mut T,
    buffer: &'a mut [u8],
}

impl<'a, T> BufferedIterator<'a, T> {
    pub fn from_iter_and_buffer(iter: &'a mut T, buffer: &'a mut [u8]) -> Self {
        BufferedIterator { iter, buffer }
    }
}

trait OptionToPostcardResult<T> {
    fn into_postcard_result(self) -> postcard::Result<T>;
}

impl OptionToPostcardResult<u8> for Option<u8> {
    fn into_postcard_result(self) -> postcard::Result<u8> {
        match self {
            Some(byte) => postcard::Result::Ok(byte),
            None => postcard::Result::Err(postcard::Error::DeserializeUnexpectedEnd),
        }
    }
}

impl<'de, T> Flavor<'de> for BufferedIterator<'de, T>
where
    T: Iterator<Item = u8>,
{
    type Remainder = ();
    type Source = BufferedIterator<'de, T>;
    fn pop(&mut self) -> postcard::Result<u8> {
        self.iter.next().into_postcard_result()
    }

    fn try_take_n(&mut self, ct: usize) -> postcard::Result<&'de [u8]> {
        let mut end_of_slice = 0;
        for i in 0..ct {
            self.buffer[i] = self.iter.next().into_postcard_result()?;
            end_of_slice += 1;
        }
        // Split the buffer so the result can use the bytes we just put into the buffer. This is necessary because
        // the 'de lifetime requires that these bytes are never reused during the whole deserialization process
        let slice = core::mem::take(&mut self.buffer);
        let (head, tail) = slice.split_at_mut(end_of_slice + 1);
        self.buffer = tail;
        postcard::Result::Ok(head)
    }

    fn finalize(self) -> postcard::Result<Self::Remainder> {
        postcard::Result::Ok(())
    }
}

pub trait TryFromIter<T, Iter, const BUF_SIZE: usize> {
    fn try_from_iter(iter: &mut Iter) -> postcard::Result<T>
    where
        Iter: Iterator<Item = u8>,
        T: DeserializeOwned,
    {
        let mut buffer = [0_u8; BUF_SIZE];
        let buffered_iter = BufferedIterator::from_iter_and_buffer(iter, &mut buffer);
        let mut deserializer = postcard::Deserializer::from_flavor(buffered_iter);
        let t = T::deserialize(&mut deserializer)?;
        postcard::Result::Ok(t)
    }
}
