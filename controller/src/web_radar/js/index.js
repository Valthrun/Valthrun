
var ws = new WebSocket('ws://192.168.1.107:6969/ws');

ws.onopen = function() {
    console.log('Connected to the WebSocket');
};

ws.onmessage = function(event) {
    // Assuming you parse the event.data to get an array of player data
    var players = JSON.parse(event.data);

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
        const size = 2048 * 2.451;
        const offset = { x: 3190, y: 3360 };
        x = (x + offset.x) / size * 100;
        y = Math.abs((y + offset.y) / size * 100 - 100);
        playerDot.style.left = `${x}%`;
        playerDot.style.top = `${y}%`;
    });

};

ws.onclose = function(event) {
    console.log('WebSocket closed:', event.code, event.reason);
};

ws.onerror = function(error) {
    console.log('WebSocket Error:', error);
};

function addPlayerDot(teamID) {
    // Create a new image element
    var playerDot = document.createElement('img');
    console.log(teamID);
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

const image = document.querySelector('.map');
image.onload = function() {
    const container = document.querySelector('.map-container');
    // Calculate the scaled width of the image
    const aspectRatio = image.naturalWidth / image.naturalHeight;
    const scaledWidth = container.offsetHeight * aspectRatio;

    // Set the container width to match the scaled width of the image
    container.style.width = `${scaledWidth}px`;
};

// Optional: If you want the container to adjust its size when the window is resized
window.addEventListener('resize', function() {
    const container = document.querySelector('.map-container');
    const aspectRatio = image.naturalWidth / image.naturalHeight;
    const scaledWidth = container.offsetHeight * aspectRatio;
    container.style.width = `${scaledWidth}px`;
});