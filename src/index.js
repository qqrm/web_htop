document.addEventListener("DOMContentLoaded", () => {
    let i = 0;

    setInterval(async () => {
        let response = await fetch("/api/cpus");

        if (response.status !== 200) {
            throw new Error(`Something went wrong!" + ${response.status}`);
        }

        let json = await response.json();

        document.body.textContent = JSON.stringify(json, null, 2);
    }, 1000)
});