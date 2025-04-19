function getCookie(name) {
    const match = document.cookie.match(new RegExp(`(^| )${name}=([^;]+)`));
    return match ? match[2] : null;
}

function getToken() {
    const token = new URLSearchParams(window.location.search).get("token");
    return token ? { token, source: "params" } : { token: getCookie("token"), source: "cookie" };
}

function setToken(token) {
    document.cookie = `token=${token}; path=/; Secure; HttpOnly`;
}

function isValidEmail(email) {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

async function fetchVersion() {
    try {
        const response = await fetch('/api/version');
        const data = await response.json();

        if (data.deployment === "beta" || data.deployment === "dev") {
            document.getElementById("version").textContent = `${data.version} (${data.deployment} build)`;
            document.getElementById("betaWarning").style.display = "block";
        } else {
            document.getElementById("version").textContent = data.version;
            document.getElementById("betaWarning").style.display = "none";
        }
    } catch (error) {
        console.error("Failed to fetch version: ", error);
    }
}

function showSwal(title, text, icon, redirectUrl = null, timer = 3000) {
    Swal.fire({
        title,
        text,
        icon,
        timer,
        timerProgressBar: true,
        showConfirmButton: false
    }).then(() => {
        if (redirectUrl) window.location.href = redirectUrl;
    });
}

document.addEventListener("DOMContentLoaded", fetchVersion);

if ("serviceWorker" in navigator) {
    navigator.serviceWorker.register("/assets/js/sw.js")
        .then(() => console.log("Service Worker registered!"))
        .catch(err => console.log("Service Worker registration failed:", err));
}