let allowedMessages = ["WebPlayersInfo", "MapInfo"];

let mapSize = 0;
let mapOffset = { x:0, y: 0 };

function messageHandlers(){
    this.WebPlayersInfo = function(data)
    {
        var players = data.players;
        // Remove all existing player dots
        var existingDots = document.querySelectorAll('.player-dot');
        existingDots.forEach(dot => dot.remove());
        // Add and position a dot for each player
        players.forEach(player => {
            var playerDot = addPlayerDot(player.team_id);

            // var rotation = player.rotation;
            var x = player.position[0]; // as a percentage of the map width
            var y = player.position[1]; // as a percentage of the map height

            // Rotate and position the player dot
            // playerDot.style.transform = `translate(-50%, -50%) rotate(${rotation}deg)`;
            x = (x + mapOffset.x) / mapSize * 100;
            y = Math.abs((y + mapOffset.y) / mapSize * 100 - 100);
            playerDot.style.left = `${x}%`;
            playerDot.style.top = `${y}%`;
        });
    }

    this.MapInfo = function(data)
    {
        if (data.name === "<empty>")
        {
            loadedMapImage.src = 'images/not_connected.png';
        }
        else
        {
            fetch(`maps/${data.name}/meta.json`)
                .then(response => response.json())
                .then(json => {
                    mapSize = json.resolution * 1024;
                    mapOffset = { x: json.offset.x, y: json.offset.y };
                });
            loadedMapImage.src = `maps/${data.name}/radar.png`;
        }
    }
}

var ws = new WebSocket('ws://192.168.1.107:6969/ws');
//var ws = new WebSocket('ws://localhost:6969/ws');

ws.onopen = function() {
    console.log('Connected to the WebSocket');
};

ws.onmessage = function(event) {
    var messageData = JSON.parse(event.data);
    let type_name = messageData.type_name;
    if (allowedMessages.indexOf(type_name)>=0)
    {
        let handler = new messageHandlers();
        handler[type_name](messageData);
    }
    else
    {
        console.error("Type not allowed: ", type_name);
    }
};

ws.onclose = function(event) {
    console.log('WebSocket closed:', event.code, event.reason);
};

ws.onerror = function(error) {
    console.log('WebSocket Error:', error);
};

function changeBackground(color) {
    document.body.style.background = color;
}

window.addEventListener("load",function() { changeBackground('black') });

function addPlayerDot(teamID) {
    // Create a new image element
    var playerDot = document.createElement('img');
    if (teamID === 3)
    {
        playerDot.src = 'images/blue_dot.png';
        playerDot.alt = 'Player';
        playerDot.className = 'player-dot';
    }
    else
    {
        playerDot.src = 'images/yellow_dot.png';
        playerDot.alt = 'Player';
        playerDot.className = 'player-dot';
    }

    // Append the player dot to the map container
    var mapContainer = document.querySelector('.map-container');
    mapContainer.appendChild(playerDot);

    return playerDot; // Return the created element for further manipulation
}

const loadedMapImage = document.querySelector('.map');
loadedMapImage.onload = function() {
    const container = document.querySelector('.map-container');
    // Calculate the scaled width of the image
    const aspectRatio = loadedMapImage.naturalWidth / loadedMapImage.naturalHeight;
    const scaledWidth = container.offsetHeight * aspectRatio;

    // Set the container width to match the scaled width of the image
    container.style.width = `${scaledWidth}px`;
};

// Optional: If you want the container to adjust its size when the window is resized
window.addEventListener('resize', function() {
    const container = document.querySelector('.map-container');
    const aspectRatio = loadedMapImage.naturalWidth / loadedMapImage.naturalHeight;
    const scaledWidth = container.offsetHeight * aspectRatio;
    container.style.width = `${scaledWidth}px`;
});