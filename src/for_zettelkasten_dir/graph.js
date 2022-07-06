let elementsData;
// let elementsData = JSON.parse(`[{
//   "data": {"id": "a"}
// }, {
//   "data": {"id": "b"}
// }, {
//   "data": {"id": "c"}
// }, {
//   "data": {"id": "e0", "source": "a", "target": "b"}
// }]`);

let cy = window.cy = cytoscape({
    container: document.getElementById('cy'),

    boxSelectionEnabled: false,
    autounselectify: true,

    layout: {
        name: 'circle'
    },

    style: [
        {
        selector: 'node',
        style: {
            'height': 20,
            'width': 20,
            'background-color': '#CCCCCC',
            'content': 'data(label)',
            'text-valign': 'center',
            'text-halign': 'center'
        }
        },
        {
        selector: 'edge',
        style: {
            'curve-style': 'haystack',
            'haystack-radius': 0,
            'width': 5,
            'opacity': 0.5,
            'line-color': '#dd0000'
        }
        }
    ],

    elements: elementsData
});