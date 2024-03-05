
// format sub package.json for dual cjs and esm support
import fs from 'fs';

const formatPackage = (source, output, type) => {
	const remove = {main: true, module: true, browser: true, types: true, exports: true};
	const json = JSON.parse(fs.readFileSync(source, 'utf-8'));
	fs.writeFileSync(output, JSON.stringify({
		...Object.fromEntries(Object.entries(json).filter(([key]) => !remove[key])),
		type:type
	}, null, 4));
}
formatPackage('./package.json', './dist/cjs/package.json', 'commonjs');
formatPackage('./package.json', './dist/mjs/package.json', 'module');
