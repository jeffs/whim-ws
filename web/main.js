import Host from './Host.js';

async function main() {
  const host = Host();
  const shell = await host.getDefaultShell();
  console.log(shell);
}

main();
