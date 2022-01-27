# Nelson

![nelson](https://user-images.githubusercontent.com/28804882/151344758-c89eb3a3-e82f-4f01-845f-e5ed8817150a.jpeg)

Nelson is a small mocking library for rust. This is currently a WIP, and uses unsafe code to achieve it's goal.

## Description

Nelson allows you to mock struct by tricking the module system. You declare a struct as you would normally do, and declare a `enum MockMystruct` in a test module. The trick is to reexport the mock struct as the real struct in test target:

```rust

#[cfg(not(test))]
pub use MyStruct;
#[cfg(test)]
pub use test::MockMyStruct as MyStruct;


pub struct MyStruct {
	// fields...
}

impl MyStruct {
	fn do_something(&self, s: &str, num: usize) -> Thingy {
	//	...
	}
}

#[cfg(test)]
mod test {
	use Nelson::Mocker;

	pub emum MockMyStruct {
		Real(super::MyStruct),
		Fake(Mocker)
	}
}

impl MockMyStruct {
	pub fn mock(mocker: Mocker) -> Self {
		Self::Fake(mocker)
	}

	fn do_something(&self, s: &str, num: usize) -> Thingy {
		match self {
			Self::Real(r) => r.do_something(thingy, usize),
			Self::Fake(m) => unsafe { m.get("do_something").call((s, num)),
		}
	}
}
```

The call to `get` and `call` is unsafe because it requires function pointer casting.

When the setup is done, you can create mocks in your tests:

```rust
use nelson::Mocker;

#[test]
fn test() {
	let mut mocker = Mocker::default();

	mocker.when::<(&str, usize), Thingy>("do_something").times(1).then(|(s, num)| Thingy);

	let my_mock = MyStruct::mock(mocker);
	my_mock.do_something("hello", 10);
}
```
