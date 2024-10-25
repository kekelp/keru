#[cfg(test)]

use crate::Watcher;

#[test]
fn example() {
    // A value representing some state in our program. This gets changed and updated according to various complicated rules, and then gets displayed to the user on the screen. 
    // Displaying it on the screen is kind of complicated as well, so we want to cleanly separate the changing logic from the displaying logic.
    let mut _value: Vec<i32> = vec![1, 2, 3];

    // This is the data that gets displayed on the screen. If we were drawing to a real screen, this would be a buffer for pixel data, not a raw String.
    // Formatting an integer to a String, converting the String into glyphs, and then rasterizing them can be a slow process. So, we want to keep track of whether the original value changed, and skip all that work when it didn't.
    let mut displayed_value = String::new();

    // We could add a separate flag `is_value_synced`, find whoever is writing the code that touches `value`, and tell them that it's their responsibility to set the flag to `false` whenever they change it. Then, when we display it, we can set the flag to `true`.
    // But even that would get annoying quickly. The people calculating the `value` really don't want to be bothered with displaying logic.
    // Instead, we ask them to wrap it into a `Watcher`, which will do this work automatically for them.

    let mut value = Watcher::new(vec![1, 2, 3]);

    // Thanks to `Deref`, the `Watcher` can be used almost in the same way as a normal `Vec`, but it automatically updates an internal flag.
    // The `Watcher` offers us a `sync` function. If the value needs syncing, it returns the current value, and resets the flag to `true`, assuming that we will display it on the screen.
    // If the value wasn't changed, it returns `None`. In this case, we can be sure that the screen still displays the correct value, and do nothing.


    // Our code for reading and displaying the value now looks like this.
    if let Some(changed_value) = value.if_changed() {
        displayed_value = format!("{:?}", changed_value);
    }
    assert_eq!(displayed_value, "[1, 2, 3]")

    // The core logic can update the value:

}


#[test]
fn new() {
    // If the value has just been created, the reader definitely won't be in sync with it, so `synced` starts as `false`. 
    let watched_char = Watcher::new('X');
    assert!(watched_char.changed == false);
}

#[test]
fn long_lived_reference() {
    let mut watched_char = Watcher::new('X');

    let mut_ref = &mut *watched_char;

    // as long as the reference exists, it can be used to modify the value without touching the changed flag.
    // however, this is not a problem, because the borrow checker will make it impossible to call set_synced() as long as any mutable references are alive.

    // causes a borrow error:
    // watched_char.set_synced();

    *mut_ref = 'Y';
    *mut_ref = 'Z';
    assert!(*watched_char == 'Z');
}

#[test]
fn false_positive_1() {
    let mut watched_string = Watcher::new("Hello".to_string());

    let _slice = &mut watched_string[..];
    // I dereferenced, but I didn't change anything.
    // It's still marked as not synced.
    // In cases like this, the compiler should normally raise a warning for unused references. 
    assert!(watched_string.changed == false);
}

#[test]
fn false_positive_2() {
    let mut watched_i32 = Watcher::new(0);

    *watched_i32 += 100;
    *watched_i32 -= 100;

    // The value was changed two times, but ended back up at the start value of 0.
    // In this case, it will be still marked as not synced.
    assert!(watched_i32.changed == false);
    assert!(*watched_i32 == 0);
}