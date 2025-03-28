function getCookie(name) {
    let match = document.cookie.match(new RegExp('(^| )' + name + '=([^;]+)'));
    return match ? match[2] : null;
}

function getToken() {
    const urlParams = new URLSearchParams(window.location.search);
    let token = urlParams.get("token");

    if (!token) {
        token = getCookie("token");
    }

    return token;
}

function setToken(token) {
    document.cookie = `token=${token}; path=/; Secure; HttpOnly`;
}

function isValidEmail(email) {
    const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailPattern.test(email);
}

async function fetchVersion() {
    try {
        await fetch('/api/version')
            .then(response => response.json())
            .then(data => {
                if (data.deployment == "beta" || data.deployment == "dev") {
                    document.getElementById("version").textContent = `${data.version} (${data.deployment} build)`;
                    document.getElementById("beta-warning").style.display = "block";
                } else {
                    document.getElementById("version").textContent = data.version;
                    document.getElementById("beta-warning").style.display = "none";
                }
            });
    } catch (error) {
        console.error("Failed to fetch version: ", error);
    }
}

function showSwal(title, text, icon, redirectUrl = null, timer = 3000) {
    Swal.fire({
        title: title,
        text: text,
        icon: icon,
        timer: timer,
        timerProgressBar: true,
        showConfirmButton: false
    }).then(() => {
        if (redirectUrl) {
            window.location.href = redirectUrl;
        }
    });
}

document.addEventListener("DOMContentLoaded", fetchVersion);

if ("serviceWorker" in navigator) {
    navigator.serviceWorker.register("/assets/js/sw.js")
        .then(() => console.log("Service Worker registered!"))
        .catch(err => console.log("Service Worker registration failed:", err));
}