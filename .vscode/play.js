var NotificationData =  {
    title: "read if cute",
    body: "<3",
    /** Whether this notification should not have a timeout */
    permanent: true,
    color: "#ff0000",
    onClick: function() {
        console.log("clicked");
    },
    onClose: function() {
        console.log("closed");
    }
}

const React = window.React;
const container = document.createElement("div");
container.id = "notification";

class Notification extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            title: props.title,
            body: props.body,
            permanent: props.permanent,
            color: props.color,
            onClick: props.onClick,
            onClose: props.onClose
        }
    }
    
    render() {
        return (
            <div className="notification" style={{backgroundColor: this.state.color}}>
                <div className="notification-title">{this.state.title}</div>
                <div className="notification-body">{this.state.body}</div>
            </div>
        )
    }
}

// render class
Notification = React.createFactory(Notification);
React.render(Notification(NotificationData), container);