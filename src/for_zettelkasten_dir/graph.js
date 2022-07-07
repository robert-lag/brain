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
            'background-color': '#969696',
            'color': '#707070',
            'content': 'data(label)',
            'text-opacity': 0.5,
            'text-valign': 'bottom',
            'text-halign': 'center',
            'font-size': '8px'
        }
        },
        {
        selector: 'edge',
        style: {
            'curve-style': 'straight',
            'width': 5,
            'opacity': 0.5,
            'line-color': '#383838'
        }
        }
    ],

    elements: elementsData
});