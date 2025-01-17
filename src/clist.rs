use libc;

use crate::other::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct clistcell {
    pub data: *mut libc::c_void,
    pub previous: *mut clistcell,
    pub next: *mut clistcell,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct clist {
    pub first: *mut clistcell,
    pub last: *mut clistcell,
    pub count: libc::c_int,
}

pub type clistiter = clistcell;
pub type clist_func =
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *mut libc::c_void) -> ()>;

/* Allocate a new pointer list */
pub unsafe fn clist_new() -> *mut clist {
    let mut lst: *mut clist = 0 as *mut clist;
    lst = malloc(::std::mem::size_of::<clist>() as libc::size_t) as *mut clist;
    if lst.is_null() {
        return 0 as *mut clist;
    }
    (*lst).last = 0 as *mut clistcell;
    (*lst).first = (*lst).last;
    (*lst).count = 0i32;
    return lst;
}
/* Destroys a list. Data pointed by data pointers is NOT freed. */
pub unsafe fn clist_free(mut lst: *mut clist) {
    let mut l1: *mut clistcell = 0 as *mut clistcell;
    let mut l2: *mut clistcell = 0 as *mut clistcell;
    l1 = (*lst).first;
    while !l1.is_null() {
        l2 = (*l1).next;
        free(l1 as *mut libc::c_void);
        l1 = l2
    }
    free(lst as *mut libc::c_void);
}
/* Some of the following routines can be implemented as macros to
be faster. If you don't want it, define NO_MACROS */
/* Inserts this data pointer before the element pointed by the iterator */
pub unsafe fn clist_insert_before(
    mut lst: *mut clist,
    mut iter: *mut clistiter,
    mut data: *mut libc::c_void,
) -> libc::c_int {
    let mut c: *mut clistcell = 0 as *mut clistcell;
    c = malloc(::std::mem::size_of::<clistcell>() as libc::size_t) as *mut clistcell;
    if c.is_null() {
        return -1i32;
    }
    (*c).data = data;
    (*lst).count += 1;
    if (*lst).first == (*lst).last && (*lst).last.is_null() {
        (*c).next = 0 as *mut clistcell;
        (*c).previous = (*c).next;
        (*lst).last = c;
        (*lst).first = (*lst).last;
        return 0i32;
    }
    if iter.is_null() {
        (*c).previous = (*lst).last;
        (*(*c).previous).next = c;
        (*c).next = 0 as *mut clistcell;
        (*lst).last = c;
        return 0i32;
    }
    (*c).previous = (*iter).previous;
    (*c).next = iter;
    (*(*c).next).previous = c;
    if !(*c).previous.is_null() {
        (*(*c).previous).next = c
    } else {
        (*lst).first = c
    }
    return 0i32;
}
/* Inserts this data pointer after the element pointed by the iterator */
pub unsafe fn clist_insert_after(
    mut lst: *mut clist,
    mut iter: *mut clistiter,
    mut data: *mut libc::c_void,
) -> libc::c_int {
    let mut c: *mut clistcell = 0 as *mut clistcell;
    c = malloc(::std::mem::size_of::<clistcell>() as libc::size_t) as *mut clistcell;
    if c.is_null() {
        return -1i32;
    }
    (*c).data = data;
    (*lst).count += 1;
    if (*lst).first == (*lst).last && (*lst).last.is_null() {
        (*c).next = 0 as *mut clistcell;
        (*c).previous = (*c).next;
        (*lst).last = c;
        (*lst).first = (*lst).last;
        return 0i32;
    }
    if iter.is_null() {
        (*c).previous = (*lst).last;
        (*(*c).previous).next = c;
        (*c).next = 0 as *mut clistcell;
        (*lst).last = c;
        return 0i32;
    }
    (*c).previous = iter;
    (*c).next = (*iter).next;
    if !(*c).next.is_null() {
        (*(*c).next).previous = c
    } else {
        (*lst).last = c
    }
    (*(*c).previous).next = c;
    return 0i32;
}
/* Deletes the element pointed by the iterator.
Returns an iterator to the next element. */
pub unsafe fn clist_delete(mut lst: *mut clist, mut iter: *mut clistiter) -> *mut clistiter {
    let mut ret: *mut clistiter = 0 as *mut clistiter;
    if iter.is_null() {
        return 0 as *mut clistiter;
    }
    if !(*iter).previous.is_null() {
        (*(*iter).previous).next = (*iter).next
    } else {
        (*lst).first = (*iter).next
    }
    if !(*iter).next.is_null() {
        (*(*iter).next).previous = (*iter).previous;
        ret = (*iter).next
    } else {
        (*lst).last = (*iter).previous;
        ret = 0 as *mut clistiter
    }
    free(iter as *mut libc::c_void);
    (*lst).count -= 1;
    return ret;
}
pub unsafe fn clist_foreach(
    mut lst: *mut clist,
    mut func: clist_func,
    mut data: *mut libc::c_void,
) {
    let mut cur: *mut clistiter = 0 as *mut clistiter;
    cur = (*lst).first;
    while !cur.is_null() {
        func.expect("non-null function pointer")((*cur).data, data);
        cur = (*cur).next
    }
}

pub unsafe fn clist_concat(mut dest: *mut clist, mut src: *mut clist) {
    if !(*src).first.is_null() {
        if (*dest).last.is_null() {
            (*dest).first = (*src).first;
            (*dest).last = (*src).last
        } else {
            (*(*dest).last).next = (*src).first;
            (*(*src).first).previous = (*dest).last;
            (*dest).last = (*src).last
        }
    }
    (*dest).count += (*src).count;
    (*src).first = 0 as *mut clistcell;
    (*src).last = (*src).first;
}

pub unsafe fn clist_nth_data(mut lst: *mut clist, mut indx: libc::c_int) -> *mut libc::c_void {
    let mut cur: *mut clistiter = 0 as *mut clistiter;
    cur = internal_clist_nth(lst, indx);
    if cur.is_null() {
        return 0 as *mut libc::c_void;
    }
    return (*cur).data;
}
#[inline]
unsafe fn internal_clist_nth(mut lst: *mut clist, mut indx: libc::c_int) -> *mut clistiter {
    let mut cur: *mut clistiter = 0 as *mut clistiter;
    cur = (*lst).first;
    while indx > 0i32 && !cur.is_null() {
        cur = (*cur).next;
        indx -= 1
    }
    if cur.is_null() {
        return 0 as *mut clistiter;
    }
    return cur;
}

pub unsafe fn clist_nth(mut lst: *mut clist, mut indx: libc::c_int) -> *mut clistiter {
    return internal_clist_nth(lst, indx);
}
