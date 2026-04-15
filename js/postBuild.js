// format sub package.json for dual cjs and esm support
import fs from 'fs';
import path from 'path';

const formatPackage = (source, output, type, extraFields = {}) => {
	try {
		// Ensure output directory exists
		const dir = path.dirname(output);
		if (!fs.existsSync(dir)) {
			fs.mkdirSync(dir, { recursive: true });
		}

		const remove = {
			main: true,
			module: true,
			browser: true,
			types: true,
			exports: true
		};

		const raw = fs.readFileSync(source, 'utf-8');
		const json = JSON.parse(raw);

		const filtered = Object.fromEntries(
			Object.entries(json).filter(([key]) => !remove[key])
		);

		const finalPackage = {
			...filtered,
			...extraFields, // allow overrides like main, exports, etc.
			type
		};

		fs.writeFileSync(output, JSON.stringify(finalPackage, null, 4));

		console.log(`✔ Generated ${output} (${type})`);
	} catch (err) {
		console.error(`✖ Error processing ${output}:`, err.message);
	}
};

// Generate CJS package.json
formatPackage(
	'./package.json',
	'./dist/cjs/package.json',
	'commonjs',
	{
		main: './index.js'
	}
);

// Generate ESM package.json
formatPackage(
	'./package.json',
	'./dist/mjs/package.json',
	'module',
	{
		module: './index.js'
	}
);
