pub struct LinkedList<T> {
    head: T,
    tail: Option<Box<LinkedList<T>>>,
}

impl<T> LinkedList<T> {
    pub(crate) fn new(item: T) -> LinkedList<T> {
        LinkedList {
            head: item,
            tail: None,
        }
    }

    pub(crate) fn insert(self, item: T) -> LinkedList<T> {
        LinkedList {
            head: item,
            tail: Some(Box::new(self)),
        }
    }

    pub(crate) fn iter<'a>(&'a self) -> LinkedListIterator<'a, T> {
        LinkedListIterator { tail: Some(&self) }
    }
}

impl<T: Clone> LinkedList<T> {
    pub(crate) fn clone_to_vec(&self) -> Vec<T> {
        self.iter().map(|item| item.clone()).collect::<Vec<T>>()
    }
}

pub(crate) struct LinkedListIterator<'a, T: 'a> {
    tail: Option<&'a LinkedList<T>>,
}

impl<'a, T> Iterator for LinkedListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tail {
            Some(val) => {
                self.tail = match &val.tail {
                    Some(tail) => Some(tail),
                    None => None,
                };
                Some(&val.head)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.tail {
            Some(_) => (1, None),
            None => (0, None),
        }
    }
}
