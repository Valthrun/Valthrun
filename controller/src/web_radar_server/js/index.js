let allowedMessages = ["WebPlayersInfo", "MapInfo"];

let mapSize = 0;
let mapOffset = { x:0, y: 0 };
let mapFloors = [];

function messageHandlers(){
    this.WebPlayersInfo = function(data)
    {
        let players = data.players;
        // Remove all existing player dots
        let existingDots = document.querySelectorAll('.player');
        existingDots.forEach(dot => dot.remove());
        // Add and position a dot for each player
        players.forEach(player => {
            let x = player.position[0];
            let y = player.position[1];
            let z = player.position[2];

            let floorOffset = { x:0, y: 0 };
            mapFloors.filter(floor => floor.zRange.min < z && floor.zRange.max > z).forEach(floor => {
                floorOffset = floor.offset;
            });

            x = ((x + mapOffset.x) / mapSize * 100) + floorOffset.x;
            y = (Math.abs(((y + mapOffset.y) / mapSize * 100 - 100)) - floorOffset.y);

            if (player.health > 0)
            {
                let playerDot = addPlayer(player.team_id, true);

                let rotation = player.rotation * -1;
                playerDot.style.left = `${x}%`;
                playerDot.style.top = `${y}%`;
                playerDot.style.transform = `translate(-50%, -50%) rotate(${rotation}deg)`;
            }
            else
            {
                let playerCross = addPlayer(player.team_id, false);
                
                playerCross.style.left = `${x}%`;
                playerCross.style.top = `${y}%`;
                playerCross.style.transform = `translate(-50%, -50%)`;
            }
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
                    mapFloors = json.floors;
                });
            loadedMapImage.src = `maps/${data.name}/radar.png`;
        }
    }
}

let ws = new WebSocket(location.origin.replace(/^http/, 'ws') + "/ws");

ws.onopen = function() {
    console.log('Connected to the WebSocket');
};

ws.onmessage = function(event) {
    let messageData = JSON.parse(event.data);
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

function addPlayer(teamID, alive) {
    // Create a new image element
    let player = document.createElement('img');
    if (teamID === 3)
    {
        if (alive)
        {
            player.src = 'images/blue_dot.png';
            player.className = 'player dot';
        }
        else
        {
            player.src = 'images/blue_cross.png'
            player.className = 'player cross';
        }

        player.alt = 'Player';
    }
    else
    {
        if (alive)
        {
            player.src = 'images/yellow_dot.png'
            player.className = 'player dot';
        }
        else
        {
            player.src = 'images/yellow_cross.png';
            player.className = 'player cross';
        }

        player.alt = 'Player';
    }

    // Append the player dot to the map container
    let mapContainer = document.querySelector('.map-container');
    mapContainer.appendChild(player);

    return player; // Return the created element for further manipulation
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

window.addEventListener('resize', function() {
    const container = document.querySelector('.map-container');
    const aspectRatio = loadedMapImage.naturalWidth / loadedMapImage.naturalHeight;
    const scaledWidth = container.offsetHeight * aspectRatio;
    container.style.width = `${scaledWidth}px`;
});