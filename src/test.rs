use crate::discord::split_message;

#[test]
fn test_split_message() {
	let mut message = "This message is shorter than the max length";
	let mut max_length = 50;
	let mut expected = vec!["This message is shorter than the max length"];
	assert_eq!(split_message(message, max_length), expected);

	message = "This test message is longer than the previous one";
	max_length = 15;
	expected = vec![
		"This test",
		"message is",
		"longer than",
		"the previous",
		"one",
	];
	assert_eq!(split_message(message, max_length), expected);

	message = "This message has a `code block` in it";
	max_length = 15;
	expected = vec!["This message", "has a", "`code block`", "in it"];
	assert_eq!(split_message(message, max_length), expected);

	message = "This message has a ```multiline\ncode block``` in it";
	max_length = 30;
	expected = vec!["This message has a", "```multiline\ncode block```", "in it"];
	assert_eq!(split_message(message, max_length), expected);

	message = "This code block is too long:\n```javascript\nfunction helloWorld() {\n\tconsole.log('Hello, world!');\n}\n```";
	max_length = 30;
	expected = vec![
		"This code block is too long:",
		"```javascript",
		"function helloWorld() {",
		"console.log('Hello, world!');",
		"}\n```",
	];
	assert_eq!(split_message(message, max_length), expected);

	message = "2 code blocks: ```js\nconsole.log('Hello, world!');\n```\n```js\nconsole.log('Goodbye, world!');\n```";
	max_length = 50;
	expected = vec![
		"2 code blocks:",
		"```js\nconsole.log('Hello, world!');\n```",
		"```js\nconsole.log('Goodbye, world!');\n```",
	];
	assert_eq!(split_message(message, max_length), expected);

	message = "1st code block fits in 1st chunk:\n```js\nconsole.log('Hello!');\n```\n```js\nconsole.log('Goodbye!');\n```";
	max_length = 70;
	expected = vec![
		"1st code block fits in 1st chunk:\n```js\nconsole.log('Hello!');\n```",
		"```js\nconsole.log('Goodbye!');\n```",
	];
	assert_eq!(split_message(message, max_length), expected);

	message = "Both of these code blocks can fit in one chunk: ```js\nconsole.log('Hello!');\n```\n```js\nconsole.log('Goodbye!');\n```";
	max_length = 70;
	expected = vec![
		"Both of these code blocks can fit in one chunk:",
		"```js\nconsole.log('Hello!');\n```\n```js\nconsole.log('Goodbye!');\n```",
	];
	assert_eq!(split_message(message, max_length), expected);
}
