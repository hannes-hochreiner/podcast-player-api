export function parseRssJsObject(obj) {
  let res = {};

  res['title'] = obj.rss.channel[0].title[0];
  res['description'] = obj.rss.channel[0].description[0];
  res['items'] = obj.rss.channel[0].item.map(itm => {
    let newItm = {};
    let guid = (itm.guid[0]._ || itm.guid[0]).trim();

    if (!guid || typeof guid !== 'string' || guid.length == 0) {
      throw new Error('Could not find GUID of item.');
    }

    newItm['guid'] = guid;
    newItm['date'] = (new Date(itm.pubDate[0])).toISOString();
    newItm['title'] = itm.title[0];

    if (itm.enclosure) {
      newItm['enclosure'] = {};
      newItm['enclosure']['url'] = itm.enclosure[0].$.url;
      newItm['enclosure']['type'] = itm.enclosure[0].$.type;
    }

    return newItm;
  });

  return res;
}