

$(document).ready(function() {
  CommandNotification.init();
  AlertNotification.init();
});

$(document).ajaxComplete(function(){AlertNotification.updateIcons()})
$(document).ajaxComplete(function(){CommandNotification.updateIcons()})

var LiveObject = function()
{
  function spanMarkup(obj, classes, content) {
    if (!content) {
      content = "";
    }
    return "<span class='" + classes.join(" ") + "' data-resource_uri='" + obj.resource_uri + "'>" + content + "</span>";
  }

  function alertIcon(obj)
  {
    return spanMarkup(obj, ['alert_indicator'])
  }

  function alertList(obj)
  {
    return spanMarkup(obj, ['alert_indicator', "alert_indicator_list"])
  }

  function alertLabel(obj)
  {
    return spanMarkup(obj, ['alert_indicator', "alert_indicator_large"])
  }

  function busyIcon(obj) {
    return spanMarkup(obj, ['notification_object_icon'])
  }

  function icons(obj) {
    return "<span class='notification_icons'>" + alertIcon(obj) + busyIcon(obj) + "</span>"
  }

  function label(obj) {
    return spanMarkup(obj, ['object_label'], obj.label)
  }

  function state(obj) {
    return spanMarkup(obj, ['object_state'], obj.state)
  }

  return {
    alertIcon: alertIcon,
    alertList: alertList,
    alertLabel: alertLabel,
    busyIcon: busyIcon,
    icons: icons,
    label: label,
    state: state
  }
}();

var Tooltip = function()
{
  function detailList(options)
  {
    var tooltip = "";
    tooltip += "<ul>"
    $.each(options.objects, function(i, obj) {
      tooltip += "<li>" + obj[options.attr] + "</li>";
    });
    tooltip += "</ul>"

    var classes = options.class || "";

    options.element.qtip({
        content: {
          text: tooltip,
          title: {
            text: options.title
          }
        },
        style: {
          classes: 'ui-tooltip-rounded ui-tooltip-shadow ' + classes
        },
        position: {
          viewport: $("div.rightpanel"),
      /*
          my: "top left",
          at: "bottom right",
          adjust: {
            method: "flip"
          }
          */
        }
    });
  }

  function sidebarMessage(title, body, persist, classes)
  {
    /* FIXME: hacky selector */
    $('div.vertical').qtip({
      content: {
        text: body,
        title: {
          text: title,
          button: persist
        }
      },
      show: {
        event: false,
        ready: true
      },
      position: {
        my: "center left",
        at: "center right",
        /* FIXME: hacky selector */
        target: $('div.vertical'),
        viewport: $('div.rightpanel'),
        adjust: {y: 40}
      },
      hide: {
        fixed: true,
        event: false
      },
      style: {
        classes: 'ui-tooltip-shadow' + " " + classes,
        /* uncomment for themeroller
        tip: 'top center',
        widget: true
        */
      },
      events: {
        render: function(event, api) {
          if (!persist) {
            setTimeout(function() {api.hide();}, 2000);
          }
        }
      }
    });
  }

  function clear(element)
  {
    if (element.qtip('api')) {
      element.qtip('api').destroy();
    }
  }

  function createGrowl(title, body_markup, persistent) {
		// Use the last visible jGrowl qtip as our positioning target
    var base_target = $('div.rightpanel');
		var target = $('.qtip.jgrowl:visible:last');

		// Create your jGrowl qTip...
		$(document.body).qtip({
			// Any content config you want here really.... go wild!
			content: {
				text: body_markup,
				title: {
					text: title,
					button: true
				}
			},
			position: {
				my: 'top right', // Not really important...
				at: (target.length ? 'bottom' : 'top') + ' right', // If target is window use 'top right' instead of 'bottom right'
				target: target.length ? target : base_target, // Use our target declared above
				adjust: { y: 5 } // Add some vertical spacing
			},
			show: {
				event: false, // Don't show it on a regular event
				ready: true, // Show it when ready (rendered)
				effect: function() { $(this).stop(0,1).fadeIn(400); }, // Matches the hide effect
				delay: 0, // Needed to prevent positioning issues
				
				// Custom option for use with the .get()/.set() API, awesome!
				persistent: persistent
			},
			hide: {
				event: false, // Don't hide it on a regular event
				effect: function(api) { 
					// Do a regular fadeOut, but add some spice!
					$(this).stop(0,1).fadeOut(400).queue(function() {
						// Destroy this tooltip after fading out
						api.destroy();

						// Update positions
						updateGrowls();
					})
				}
			},
			style: {
				classes: 'jgrowl ui-tooltip-red ui-tooltip-rounded', // Some nice visual classes
				tip: false // No tips for this one (optional ofcourse)
			},
			events: {
				render: function(event, api) {
					// Trigger the timer (below) on render
					timer.call(api.elements.tooltip, event);
				}
			}
		})
		.removeData('qtip');
	};

	function updateGrowls() {
		// Loop over each jGrowl qTip
		var each = $('.qtip.jgrowl:not(:animated)');
		each.each(function(i) {
			var api = $(this).data('qtip');

			// Set the target option directly to prevent reposition() from being called twice.
			api.options.position.target = !i ? $(document.body) : each.eq(i - 1);
			api.set('position.at', (!i ? 'top' : 'bottom') + ' right');
		});
	};

	// Setup our timer function
	function timer(event) {
		var api = $(this).data('qtip'),
			lifespan = 5000; // 5 second lifespan
		
		// If persistent is set to true, don't do anything.
		if(api.get('show.persistent') === true) { return; }

		// Otherwise, start/clear the timer depending on event type
		clearTimeout(api.timer);
		if(event.type !== 'mouseover') {
			api.timer = setTimeout(api.hide, lifespan);
		}
	}

  function hide(event) {
		var api = $(this).data('qtip');
    api.hide();
  }

	// Utilise delegate so we don't have to rebind for every qTip!
	$(document).delegate('.qtip.jgrowl', 'mouseover mouseout', timer);
	$(document).delegate('.qtip.jgrowl', 'click', hide);
  
  return {
    sidebarMessage: sidebarMessage,
    detailList: detailList,
    clear: clear,
    createGrowl: createGrowl
  }
}();

var CommandNotification = function() {
  var incomplete_commands = {};
  var incomplete_jobs = {};
  var read_locks = {}
  var write_locks = {}
  var fast_update_interval = 5000;

  function updateBusy()
  {
    var command_ids = []
    $.each(incomplete_commands, function(i, command) {
      command_ids.push(command.id);
    });
    if (command_ids.length == 0) {
      $('#notification_icon_jobs').slideUp()
      return false;
    } else {
      if (command_ids.length == 1) {
        setTimeout(update, fast_update_interval);
      }
      Tooltip.detailList({element: $('#notification_icon_jobs'),
          title: command_ids.length + " commands running:", objects: incomplete_commands, attr: 'message'});
      $('#notification_icon_jobs').slideDown()
      return true;
    }
  }

  function init()
  {
    // Populate incomplete_commands
    Api.get("/api/command/", {complete: false, limit: 0}, success_callback = function(data) {
      var commands = data.objects;
      $.each(commands, function(i, command) {
        begin(command);
      });
    }, error_callback = undefined);
  }

  function begin(command)
  {
    if (command.complete) {
      // If something finished before it started, no need to track
      // its state
      notify(command);
      return;
    }
    if (incomplete_commands[command.id]) {
      // If something gets double inserted (e.g. if init is slow and
      // picks up a user action)
      return;
    }
    notify(command);
    updateJobs(command);
    incomplete_commands[command.id] = command;
    updateBusy()
  }

  function notify(command)
  {
    var header;
    var theme;
    var persist;
    if (!command.complete) {
      /*header = "Command started";
      theme = 'job_success';
      persist = false;*/
      return
    } else if (command.cancelled) {
      /*
      header = "Command cancelled";
      theme = '';
      persist = true;
      Tooltip.message("<a class='navigation' href='command/" + command.id + "/'>" + header + "</a>", command.message, persist, theme)
      */
    } else if (command.errored) {
      var body = command.message;
      body += "&nbsp;<a class='navigation' href='command/" + command.id + "/'>Details...</a>";
      Tooltip.createGrowl("Command failed", body, true);
    } else {
      /*
      header = "Command complete";
      theme = 'ui-tooltip-green';
      persist = false;
      Tooltip.message("<a class='navigation' href='command/" + command.id + "/'>" + header + "</a>", command.message, persist, theme)
      */
    }

  }

  function update()
  {
    var command_ids = []
    $.each(incomplete_commands, function(i, command) {
      command_ids.push(command.id);
    });

    if (command_ids.length == 0) {
      return;
    }

    Api.get("/api/command/", {id__in: command_ids, limit: 0}, success_callback = function(data) {
      var commands = data.objects;
      var busy = true;
      $.each(commands, function(i, command) {
        if (command.complete) {
          notify(command);
          updateJobs(command);
          delete incomplete_commands[command.id]
          busy = updateBusy();
        }
      });
      if (busy) {
        setTimeout(update, fast_update_interval);
      }
    }, error_callback = undefined, blocking = false);
  }

  function updateJobs(command)
  {
    var query_jobs = [];
    $.each(command.jobs, function(i, job_uri) {
      var tokens = job_uri.split("/")
      var job_id = tokens[tokens.length - 2]
      if (command.complete) {
        job = incomplete_jobs[job_id]
        if (job) {
          completeJob(job)
        }
      } else {
        if (incomplete_jobs[job_id] == undefined) {
          query_jobs.push(job_id)
        }
      }
    });

    if (query_jobs.length == 0) {
      return;
    }

    Api.get("/api/job/", {id__in: query_jobs, limit: 0}, success_callback = function(data) {
      var jobs = data['objects']
      $.each(jobs, function(i, job) {
        startJob(job);
      });
    }, error_callback = undefined, blocking = false);
  }

  function updateIcon(uri, element, completing) {
    var obj_read_locks = read_locks[uri]
    var obj_write_locks = write_locks[uri]
    if (obj_write_locks && objectLength(obj_write_locks) > 0) {
      element.show();
      element.addClass('busy_icon');
      var jobs = {}
      $.each(obj_write_locks, function(job_id, x) {
        jobs[job_id] = incomplete_jobs[job_id]
      });
      Tooltip.detailList(
          {element: element, title: "Ongoing operations: ", objects: jobs, attr: 'description'});
    } else if (obj_read_locks && objectLength(obj_read_locks) > 0) {
      element.show();
      element.addClass('locked_icon');
      var jobs = {}
      $.each(obj_read_locks, function(job_id, x) {
        jobs[job_id] = incomplete_jobs[job_id]
      });
      Tooltip.detailList(
          {element: element, title: "Locked by pending operations: ", objects: jobs, attr: 'description'});
    } else {
      element.removeClass('locked_icon');
      element.removeClass('busy_icon');
    }
  }

  function updateIcons(){
    $('notification_object_icon').each(function() {
      var uri = $(this).data('resource_uri')
      updateIcon(uri, $(this));
    });
  }

  function startJob(job) {
    if (incomplete_jobs[job.id]) {
      throw "Job " + job.id + " started twice"
    }
    incomplete_jobs[job.id] = job

    var uris = {};
    $.each(job.read_locks, function(i, lock) {
      var uri = lock.locked_item_uri;
      if (uri) {
        if (!read_locks[uri]) {
          read_locks[uri] = {}
        }
        read_locks[uri][job.id] = 1

        uris[uri] = 0;
      }
    });
    $.each(job.write_locks, function(i, lock) {
      var uri = lock.locked_item_uri;
      if (uri) {
        if (!write_locks[uri]) {
          write_locks[uri] = {}
        }
        write_locks[uri][job.id] = 1
        uris[uri] = 0;
      }
    });

    $.each(uris, function(uri, x) {
      $(".notification_object_icon[data-resource_uri='" + uri + "']").each(function() {
        updateIcon(uri, $(this), true);
      });
      $(".transition_buttons[data-resource_uri='" + uri + "']").each(function() {
        $(this).html("")
      });
    });
  }

  function updateObject(uri)
  {
    Api.get(uri, {},
      success_callback = function(obj) {
      $(".object_label[data-resource_uri='" + uri + "']").each(function() {
        $(this).html(obj.label);
      });
      $(".object_state[data-resource_uri='" + uri + "']").each(function() {
        $(this).html(obj.state);
      });

      $(".transition_buttons[data-resource_uri='" + uri + "']").each(function() {
        $(this).html(stateTransitionButtons(obj));
      });
    },
    error_callback = {404: function() {
      // The object has gone away
      // TODO: handle removing it from its container (e.g. row from table)
      $(".object_label[data-resource_uri='" + uri + "']").each(function() {
        $(this).html("");
      });
      $(".object_state[data-resource_uri='" + uri + "']").each(function() {
        $(this).html("");
      });

      $(".transition_buttons[data-resource_uri='" + uri + "']").each(function() {
        $(this).html("");
      });
    }},
    blocking = false);
  }

  function completeJob(job) {
    delete incomplete_jobs[job.id]

    var uris = {};
    $.each(job.read_locks, function(i, lock) {
      var uri = lock.locked_item_uri;
      if (uri) {
        delete read_locks[uri][job.id]
        uris[uri] = true;
      }
    });
    $.each(job.write_locks, function(i, lock) {
      var uri = lock.locked_item_uri;
      if (uri) {
        delete write_locks[uri][job.id]
        uris[uri] = true;
      }
    });

    $.each(uris, function(uri, x) {
      if (objectLength(read_locks[uri]) == 0 && objectLength(write_locks[uri]) == 0) {
        updateObject(uri)
      }
      $(".notification_object_icon[data-resource_uri='" + uri + "']").each(function() {
        updateIcon(uri, $(this));
      });
    });
  }

  return {
    init: init,
    begin: begin,
    updateIcons: updateIcons
  }
}();


var AlertNotification = function() {
  var slow_poll_interval = 5000;
  var active_alerts = {};
  var alert_effects = {};
  var initialized = false;

  function init()
  {
    if (initialized) {
      return;
    }
    update(true);
    initialized = true;
  }

  function update(initial_load)
  {
    Api.get("/api/alert/", {active: true, limit: 0}, success_callback = function(data) {
      var seen_alerts = {}
      var new_alerts = []
      var resolved_alerts = []

      $.each(data.objects, function(i, alert_info) {
        seen_alerts[alert_info.id] = true
        if (!active_alerts[alert_info.id]) {
          activateAlert(alert_info);
          new_alerts.push(alert_info)
        }
      });

      $.each(active_alerts, function(i, alert_info) {
        if (!seen_alerts[alert_info.id]) {
          deactivateAlert(alert_info)
          resolved_alerts.push(alert_info)
        }
      });

      if (!initial_load) {
        if (new_alerts.length == 1 && resolved_alerts.length == 0) {
          Tooltip.sidebarMessage("New alert", new_alerts[0].message, false, 'ui-tooltip-red');
        } else if (new_alerts.length == 0 && resolved_alerts.length == 1) {
          /*
          Tooltip.message("Alert cleared", resolved_alerts[0].message, false, 'ui-tooltip-green');
          */
        } else if (new_alerts.length > 0 && resolved_alerts.length == 0) {
          Tooltip.sidebarMessage(new_alerts.length + " alerts active", " ", false, 'ui-tooltip-red');
        } else if (new_alerts.length == 0 && resolved_alerts.length > 0) {
          /*
          Tooltip.message(resolved_alerts.length + " alerts resolved", " ", false, 'ui-tooltip-green');
          */
        } else if (new_alerts.length > 0 && resolved_alerts.length > 0) {
          Tooltip.sidebarMessage("Alerts",
              new_alerts.length + " new alerts, " + resolved_alerts.length + " alerts resolved",
              false, 'ui-tooltip-red');
        }
      }

      setTimeout(update, slow_poll_interval);
    }, error_callback = undefined, blocking = false);
  }

  function updateSidebar()
  {
    var active_alert_count = objectLength(active_alerts);
    if (active_alert_count == 0) {
      $('#notification_icon_alerts').slideUp()
    } else {
      $('#notification_icon_alerts').slideDown()
      $('#notification_icon_alerts').effect('pulsate', {times: 3})
      Tooltip.detailList({element: $('#notification_icon_alerts'),
          title: active_alert_count + " alerts active:", objects: active_alerts, attr: 'message', class: 'ui-tooltip-red'});
    }
  }

  function activateAlert(alert_info) {
    if (active_alerts[alert_info.id]) {
      throw "Alert " + alert_info.id + " activated twice"
    }

    active_alerts[alert_info.id] = alert_info

    $.each(alert_info.affected, function(i, effectee) {
      if (!alert_effects[effectee.resource_uri]) {
        alert_effects[effectee.resource_uri] = {}
      }
      alert_effects[effectee.resource_uri][alert_info.id] = 1
      $(".alert_indicator[data-resource_uri='" + effectee.resource_uri + "']").each(function() {
        updateIcon($(this));
      });
    });

    updateSidebar();
  }

  function deactivateAlert(alert_info) {
    if (active_alerts[alert_info.id] == null) {
      throw "Alert " + alert_info.id + " finished but not in active_alerts"
    }

    delete active_alerts[alert_info.id]

    $.each(alert_info.affected, function(i, effectee) {
      delete alert_effects[effectee.resource_uri][alert_info.id]
      $(".alert_indicator[data-resource_uri='" + effectee.resource_uri + "']").each(function() {
        updateIcon($(this));
      });
    });

    updateSidebar();
  }

  function updateIcon(element)
  {
    var uri = element.data('resource_uri')
    var effects = alert_effects[uri]

    effect_count = objectLength(effects)
    if (element.hasClass('alert_indicator_list')) {
      var list_markup = "<ul>";
      if (effect_count > 0) {
        $.each(effects, function(alert_id, x) {
          var alert_info = active_alerts[alert_id]
          list_markup += "<li>" + alert_info.message + "</li>"
        });
        list_markup += "</ul>";
        element.html(list_markup);
        element.removeClass('alert_indicator_inactive');
        element.addClass('alert_indicator_active');
      } else {
        element.html("No alerts");
        element.removeClass('alert_indicator_active');
        element.addClass('alert_indicator_inactive');
      }
    } else {
      if (effect_count > 0) {
        if (element.hasClass('alert_indicator_large')) {
          var text = effect_count + ' alert';
          if (effect_count > 1) {
            text += "s";
          }
          element.html(text)
        }
        element.addClass('alert_indicator_active');
        element.removeClass('alert_indicator_inactive');
        
        var alerts = {};
        $.each(effects, function(alert_id, x) {
          alerts[alert_id] = active_alerts[alert_id]
        });
        Tooltip.detailList({element: element, title: "Alerts", objects: alerts, attr: 'message', class: 'ui-tooltip-red'});
      } else {
        if (element.hasClass('alert_indicator_large')) {
          element.html('No alerts')
        }
        element.addClass('alert_indicator_inactive');
        element.removeClass('alert_indicator_active');
        element.attr('title', 'No alerts');
        Tooltip.clear(element)
      }
    }
  }

  function updateIcons()
  {
    $('.alert_indicator').each(function() {
      updateIcon($(this));
    });
  }

  return {
    init: init,
    updateIcons: updateIcons
  }
}();

objectLength = function(obj) {
  var count = 0;
  for (var k in obj) {
    if (obj.hasOwnProperty(k)) {
      ++count;
    }
  }
  return count;
}

