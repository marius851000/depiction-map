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
  document.getElementById("status").textContent = "status: " + content;
}

updateStatus("Loading web page");

let map = L.map("map", {
  fullscreenControl: true,
  fullscreenControlOptions: {
    position: "topleft",
  },
}).setView([12.9, 15.3], 3);

L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
  maxZoom: 19,
  attribution:
    '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

var sidebar = L.control.sidebar("sidebar", {
  position: "right",
  closeButton: false, //TODO: would be nice to close (and re-open) this
  autoPan: false,
});

map.addControl(sidebar);

let xhr = new XMLHttpRequest();

let saved_answer = null;
let existing_layer = null;

function refresh_map_if_needed() {
  if (existing_layer != null) {
    refresh_map();
  }
}

xhr.onreadystatechange = function () {
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
};

function refresh_map() {
  console.log("refresh_map called");

  if (existing_layer != null) {
    map.removeLayer(existing_layer);
  }

  const ignore_exhibits = !document.getElementById("include_exhibits").checked;
  const missing_image_only =
    document.getElementById("missing_image_only").checked;

  let markers = L.markerClusterGroup({
    maxClusterRadius: 30,
  });

  existing_layer = markers;

  for (entry of Object.entries(saved_answer)) {
    entry = entry[1];
    //console.log(entry);

    if (entry["pos"] == null) {
      continue;
    }

    if (ignore_exhibits && entry["is_in_exhibit"] == true) {
      continue;
    }

    if (missing_image_only && entry["image"] != null) {
      continue;
    }

    let name = entry["name"];
    if (name == null) {
      name = "no/unknown name";
    }

    //TODO: generate HTML on-the-fly (for faster processing time)

    popupHTML = "<b>" + escapeHtml(name) + "</b><br />";

    location_name = entry["location_name"];
    nature_name = entry["nature"];
    if (location_name != null || nature_name != null) {
      popupHTML += "<i>";
      if (nature_name != null) {
        popupHTML += escapeHtml(nature_name);
        if (location_name != null) {
          popupHTML += " — ";
        }
      }
      if (location_name != null) {
        popupHTML += escapeHtml(location_name);
      }
      popupHTML += "</i><br />";
    }

    // image
    //TODO: (more server-side) some images are TIF that doesn’t display in browser
    if (entry["image"] != null) {
      imageURL = entry["image"]["url"];
      imageCreditUrl = entry["image"]["credit_url"];

      popupHTML +=
        '<img src="' + escapeHtml(imageURL) + '" class="embed-image"/><br />';
      imageSourceText = entry["image"]["credit_text"];

      if (imageCreditUrl != null) {
        if (imageSourceText == null) {
          imageSourceText = "image source";
        }
        popupHTML +=
          '<p><a href="' +
          escapeHtml(imageCreditUrl) +
          '">' +
          escapeHtml(imageSourceText) +
          "</a></p>";
      } else {
        if (imageSourceText == null) {
          imageSourceText = "image source somehow unknwown.";
        }
        popupHTML += "<p>" + escapeHtml(imageSourceText) + "</p>";
      }
    } else {
      popupHTML += "<p><i>No image</i></p><br />";
    }

    // link
    popupHTML +=
      '<a href="' + escapeHtml(entry["source_url"]) + '">Data source</a>';

    let marker = L.marker(entry["pos"], {}).bindPopup(popupHTML, {
      maxWidth: "auto",
    });

    markers.addLayers(marker);
  }

  map.addLayer(markers);
}

updateStatus("fetching datas");
xhr.open("GET", "/depiction/dragon.json", true);
xhr.setRequestHeader("Accept", "application/json");
xhr.send();

sidebar.show();

document.getElementById("noscript").remove();
