import init, { add } from './pkg/variables.js';

async function main() {
    init().then(() => {
        console.log(add(4,5));
      });
}

main();