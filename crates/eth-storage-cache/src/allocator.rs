use crate::Page;

struct FreeNode {
    page: Page,
    next: Option<Box<FreeNode>>,
}
pub struct PageAllocator {
    free_list: Option<Box<FreeNode>>,
}

impl PageAllocator {
    pub fn new() -> Self {
        PageAllocator { free_list: None }
    }

    pub fn allocate(&mut self) -> Page {
        if let Some(free_node) = self.free_list.take() {
            let page = free_node.page;
            self.free_list = free_node.next;
            page
        } else {
            Page::new()
        }
    }

    pub fn free(&mut self, page: Page) {
        let recycle_node = FreeNode {
            page,
            next: self.free_list.take(),
        };
        self.free_list = Some(Box::new(recycle_node));
    }

    pub fn free_count(&self) -> usize {
        let mut count = 0;
        let mut node = self.free_list.as_deref();
        while let Some(n) = node {
            count += 1;
            node = n.next.as_deref();
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_then_free_increments_free_count() {
        let mut allocator = PageAllocator::new();
        assert_eq!(allocator.free_count(), 0);
        let page = allocator.allocate();
        assert_eq!(allocator.free_count(), 0);
        allocator.free(page);
        assert_eq!(allocator.free_count(), 1);
    }

    #[test]
    fn free_then_allocate_returns_same_page() {
        let mut allocator = PageAllocator::new();
        let page = allocator.allocate();
        let ptr = page.as_ptr();
        allocator.free(page);
        let new_page_ptr = allocator.allocate().as_ptr();
        assert_eq!(ptr, new_page_ptr);
    }

    #[test]
    fn allocate_returns_pages_in_lifo_order() {
        let mut allocator = PageAllocator::new();
        let a = allocator.allocate();
        let b = allocator.allocate();
        let c = allocator.allocate();
        let a_ptr = a.as_ptr();
        let b_ptr = b.as_ptr();
        let c_ptr = c.as_ptr();

        allocator.free(a);
        allocator.free(b);
        allocator.free(c);

        assert_eq!(allocator.allocate().as_ptr(), c_ptr);
        assert_eq!(allocator.allocate().as_ptr(), b_ptr);
        assert_eq!(allocator.allocate().as_ptr(), a_ptr);
    }
}
