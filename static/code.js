// Why isn’t that part of Javascript... I’m not surprised CSP exist with that.
function escapeHtml(unsafe) {
    return unsafe
         .replace(/&/g, "&amp;")
         .replace(/</g, "&lt;")
         .replace(/>/g, "&gt;")
         .replace(/"/g, "&quot;")
         .replace(/'/g, "&#039;");
}

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
            updateStatus("processing...");
            refresh_map();
            updateStatus("ready to use!");
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

        let name = entry["name"];
        if (name == null) {
            name = "no/unknown name";
        }



        //TODO: generate HTML on-the-fly (for faster processing time)

        popupHTML = "<b>" + escapeHtml(name) + "</b><br />";

        //TODO: determine nature in the server-side code to display it here like in the original
        location_name = entry["location_name"]
        if (location_name != null) {
            popupHTML += "<i>" + escapeHtml(location_name) + "</i>";
        }

        // image
        //TODO:
        /*imageURL = null
        imageCredit = null
        if (qid in extra_images) {
            imageURL = "./extra/" + qid + "." + extra_images[qid][0];
            imageCredit = "image from <a href=\"" + extra_images[qid][1] + "\">here</a>"
        } else if (entry["image"] != undefined) {
            imageURL = escapeHtml(entry["image"]["value"]);
            imageCredit = "image from Wikimedia commons";
        }
        if (imageURL != null) {
            popupHTML += "<img src=\"" + imageURL + "\" class=\"embed-image\"/><br /><p>" + imageCredit + "</p><br />";
        } else {
            popupHTML += "<p><i>No image</i></p><br />"
            console.log(entry["itemLabel"]["value"] + " ( https://wikidata.org/wiki/" + qid + " ) have no image")
        }*/

        // link
        popupHTML += "<a href=\"" + escapeHtml(entry["source_url"]) + "\">Data source</a>"

        let marker = L.marker(entry["pos"], {})
            .bindPopup(popupHTML, {
                maxWidth: "auto",
            });

        markers.addLayers(marker);
    }

    map.addLayer(markers)
}

updateStatus("fetching datas");
xhr.open("GET", "/depiction/dragon.json", true);
xhr.setRequestHeader("Accept", "application/json");
xhr.send();