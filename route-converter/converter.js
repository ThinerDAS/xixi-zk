"F12, Copy, Replay_Route, Beat_Boss, Download";
(function () {
    var xixi_mapping_s = JSON.parse(LZString.decompressFromBase64("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"));
    var old_trigger = events.prototype.trigger;
    events.prototype.trigger = function (x, y, callback) {
        var coord = [x, y, core.status.floorId];
        if (!flags.tile_trace) flags.tile_trace = [];
        flags.tile_trace.push(coord);
        old_trigger.apply(this, arguments);
    }
    core.summarize = function (dryonly) {
        var tt = flags.tile_trace;
        var res = [];
        for(var i=0; i<tt.length; i++) {
            var coord = tt[i];
            var key = [coord[0], coord[1], parseInt(coord[2].slice(1))-1];
            var index = xixi_mapping_s.indexOf(key.join('|'));
            if (index < 0) continue;
            if (index == 0) continue;
            if (res.indexOf(index) >= 0) continue;
            res.push(index);
        }
        if (res.indexOf(1) < 0) { res.push(1); }
        if (res.length != xixi_mapping_s.length - 1) { alert("Warning short route"); }
        if (!dryonly) { core.download(
            "route_"+core.formatDate2()+".txt", JSON.stringify(res)); }
        return res;
    }
    var old_win = events.prototype.win;
    events.prototype.win = function () {
        core.summarize();
        alert("Route downloaded");
        old_win.apply(this, arguments);
    }
})();
