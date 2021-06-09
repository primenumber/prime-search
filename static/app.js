async function nth_prime(n) {
  const response = await fetch('./api/nth_prime', {
    method: 'POST',
    body: `n=${n}`
  });
  return response.json();
}

function get_nth_prime() {
  const n = parseInt(document.getElementById("n").value);
  nth_prime(n).then(data => {
    if (data["status"] === "ok") {
      document.getElementById("result").textContent = data["result"];
    } else {
      document.getElementById("result").textContent = "Error: " + data["message"];
    }
  });
}
