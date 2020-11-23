async function main() {
  let { greet } = await import('./hello.js');

  const header = document.createElement('h1');
  header.append(greet('there'));

  const greeter = document.createElement('button');
  greeter.append('Greet me');
  greeter.addEventListener('click', function () {
    header.innerText = greet('clicked');
  });

  const reloader = document.createElement('button');
  reloader.append('Reload module');
  reloader.addEventListener('click', async () => {
    console.log('reloading');
    greet = (await import('./hello.js')).greet;
    console.log('greet === greet? ', greet === greet);  // true :-(
  });

  document.body.append(header);
  document.body.append(greeter);
  document.body.append(reloader);
}

main();
