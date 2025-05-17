function updateStatus(content) {
    document.getElementById("status").textContent = "status: " + content
}

updateStatus("Loading web page");

let map = L.map('map', {
    fullscreenControl: true,
    fullscreenControlOptions: {
        position: 'topleft'
    }
}).setView([12.9, 15.3], 3);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
}).addTo(map);



let xhr = new XMLHttpRequest();

let saved_answer = null;
let existing_layer = null;

xhr.onreadystatechange = function() {
    if (this.readyState == 4) {
        if (this.status == 200) {
            saved_answer = JSON.parse(this.responseText);

            // nature which are typically not associated with coordinates
            refresh_map();
            updateStatus("ready");
        } else {
            alert("No valid response received");
            updateStatus("failure");
        }
    }
}

function refresh_map() {
    console.log("refresh_map called");
    if (existing_layer != null) {
        map.removeLayer(existing_layer);
    }

    //const ignore_exhibits = !document.getElementById("include_exhibits").checked;
    const ignore_exhibits = false; //TODO:

    let markers = L.markerClusterGroup({
        "maxClusterRadius": 30,
    });

    existing_layer = markers;

    for (entry of Object.entries(saved_answer)) {
        entry = entry[1];
        console.log(entry);
        if (ignore_exhibits && entry["is_in_exhibit"] == true) {
            continue;
        }

        let marker = L.marker(entry["pos"], {});

        markers.addLayers(marker);
    }

    map.addLayer(markers)
}

updateStatus("fetching datas");
xhr.open("GET", "/depiction/dragon.json", true);
xhr.setRequestHeader("Accept", "application/json");
xhr.send();