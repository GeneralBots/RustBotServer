let items  = FIND "gb.rob", "ACTION=EMUL1"
FOR EACH item IN items  

    PRINT item.company
    let website = WEBSITE OF item.company
    PRINT website
    
    WAIT 10
    let page = GET website

    let prompt = "Create a website for " + item.company + " with the following details: " + page

    let alias = LLM "Return a single word for " + item.company + " like a token, no spaces, no special characters, no numbers, no uppercase letters."

    CREATE SITE item.company + "bot", item.company, website, "site", prompt 

    let to = item.emailcto
    let subject = "Simulador criado " + item.company
    let body = "O simulador " + item.company + " foi criado com sucesso. Acesse o site: " + item.company + "bot"

	CREATE_DRAFT to, subject, body

NEXT item
