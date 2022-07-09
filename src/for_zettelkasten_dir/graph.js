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

	boxSelectionEnabled: true,
    autounselectify: false,
	idealEdgeLength: 200,

    layout: {
        name: 'cose'
    },

	style: cytoscape.stylesheet()
		.selector('node')
			.css({
				'height': 'mapData(weight, 0, 30, 20, 60)',
				'width': 'mapData(weight, 0, 30, 20, 60)',
				'color': '#707070',
				'text-opacity': 1,
				'text-valign': 'bottom',
				'text-halign': 'center',
				'font-size': '10px',
			})
		.selector('node:unselected')
			.css({
				'background-color': '#969696',
			})
		.selector('node:selected, node:grabbed')
			.css({
				'content': 'data(label)',
				'background-color': '#8BA7BD',
			})
		.selector('edge')
			.css({
				'curve-style': 'straight',
				'width': 8,
				'opacity': 0.5,
				'line-color': '#383838'
			}),

    elements: elementsData
});